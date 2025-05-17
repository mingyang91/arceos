//! Task waker implementation.

#[cfg(feature = "multitask")]
use alloc::boxed::Box;

#[cfg(feature = "timer")]
use axhal::time::TimeValue;
#[cfg(feature = "timer")]
use core::sync::atomic::{AtomicU64, Ordering};

/// A simple waker that calls the given callback when woken.
pub struct SimpleWaker<F: Fn() + Send + Sync + Clone + 'static>(F);

impl<F: Fn() + Send + Sync + Clone + 'static> SimpleWaker<F> {
    /// Creates a new waker that calls the given callback.
    pub fn new(f: F) -> Self {
        Self(f)
    }

    /// Converts this waker into a [`Waker`].
    #[cfg(feature = "multitask")]
    pub fn into_waker(self) -> Waker {
        let ptr = Box::into_raw(Box::new(self));
        let raw_waker = RawWaker::new(ptr as *const (), &SimpleWaker::<F>::VTABLE);
        unsafe { Waker::from_raw(raw_waker) }
    }

    #[cfg(feature = "multitask")]
    #[inline]
    fn from_ptr<'a>(ptr: *const ()) -> &'a Self {
        unsafe { &*(ptr as *const Self) }
    }

    #[cfg(feature = "multitask")]
    #[inline]
    fn into_box(ptr: *const ()) -> Box<Self> {
        unsafe { Box::from_raw(ptr as *mut Self) }
    }

    #[cfg(feature = "multitask")]
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        // Clone
        |ptr| {
            let waker = SimpleWaker::<F>::from_ptr(ptr);
            let cloned = Box::new(SimpleWaker(waker.0.clone()));
            let ptr = Box::into_raw(cloned);
            RawWaker::new(ptr as *const (), &SimpleWaker::<F>::VTABLE)
        },
        // Wake
        |ptr| {
            let waker = SimpleWaker::<F>::into_box(ptr);
            (waker.0)();
        },
        // Wake by reference
        |ptr| {
            let waker = SimpleWaker::<F>::from_ptr(ptr);
            (waker.0)();
        },
        // Drop
        |ptr| {
            let _ = SimpleWaker::<F>::into_box(ptr);
        },
    );
}

// Timer-based wakers
#[cfg(feature = "timer")]
mod timer_waker {
    use super::*;
    use crate::TimerEvent;
    use spin::Mutex;

    // Unique ID for timer events
    static TIMER_TICKET_ID: AtomicU64 = AtomicU64::new(1);

    struct WakerTimerEvent {
        ticket_id: u64,
        waker: Waker,
    }

    impl TimerEvent for WakerTimerEvent {
        fn callback(self, _now: TimeValue) {
            self.waker.wake();
        }
    }

    // Global timer list with proper synchronization
    static TIMER_LIST: Mutex<Option<crate::TimerList<WakerTimerEvent>>> = Mutex::new(None);

    /// Initializes the timer-based waker subsystem.
    pub fn init_timer_waker() {
        let mut timer_list = TIMER_LIST.lock();
        if timer_list.is_none() {
            *timer_list = Some(crate::TimerList::new());
        }
    }

    /// Sets a waker to be woken at the specified deadline.
    ///
    /// Only one timer can be active for each waker at a time.
    pub fn wake_at(deadline: TimeValue, waker: Waker) {
        // trace!("Setting waker to wake at {:?}", deadline);
        let ticket_id = TIMER_TICKET_ID.fetch_add(1, Ordering::AcqRel);

        let mut timer_list_guard = TIMER_LIST.lock();
        if let Some(timer_list) = timer_list_guard.as_mut() {
            timer_list.set(deadline, WakerTimerEvent { ticket_id, waker });
        }
    }

    /// Processes pending timer events.
    ///
    /// This should be called periodically, e.g., from the timer interrupt handler.
    pub fn check_timer_events() {
        let now = axhal::time::monotonic_time();

        // Process all pending events
        loop {
            // Get an event to process
            let event_to_process = {
                let Some(mut timer_list_guard) = TIMER_LIST.try_lock() else {
                    debug!("Another timer event is being processed");
                    return;
                };
                if let Some(timer_list) = timer_list_guard.as_mut() {
                    timer_list.expire_one(now)
                } else {
                    None
                }
            };

            // Process the event outside the lock
            match event_to_process {
                Some((_deadline, event)) => {
                    // debug!("Waking waker with ticket id {}", event.ticket_id);
                    event.callback(now)
                }
                None => break,
            }
        }
    }
}

#[cfg(feature = "timer")]
pub use self::timer_waker::*;

#[cfg(feature = "multitask")]
mod task_waker {
    use super::*;
    use spin::Mutex;

    /// A waker for async tasks that wakes up the current axtask
    #[cfg(feature = "multitask")]
    pub fn current_task_waker() -> Waker {
        // Use a callback-based waker that calls yield_now as a simpler alternative
        SimpleWaker::new(|| {
            trace!("Waking current task by yielding");
            // This will allow the task to be rescheduled
            axtask::yield_now();
        })
        .into_waker()
    }

    /// A waker that is tied to a specific callback function.
    ///
    /// It can be used to wake up the current task from other contexts.
    pub struct AsyncTask {
        inner: Mutex<AsyncTaskInner>,
    }

    struct AsyncTaskInner {
        waker: Option<Waker>,
    }

    impl AsyncTask {
        /// Creates a new async task.
        pub fn new() -> Self {
            Self {
                inner: Mutex::new(AsyncTaskInner { waker: None }),
            }
        }

        /// Sets the waker for this task.
        pub fn set_waker(&self, waker: &Waker) {
            let mut inner = self.inner.lock();
            if let Some(old_waker) = &inner.waker {
                if old_waker.will_wake(waker) {
                    return;
                }
            }
            inner.waker = Some(waker.clone());
        }

        /// Wakes the task if there's a registered waker.
        pub fn wake(&self) {
            let waker = {
                let mut inner = self.inner.lock();
                inner.waker.take()
            };
            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }
}
