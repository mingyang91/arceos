//! Task waker implementation for multitasking environments.

use alloc::sync::Arc;
use core::task::Waker;
use spin::Mutex;

use axtask::{AxTaskRef, TaskState};

/// A waker for async tasks that wakes up a specific task.
pub struct TaskWaker {
    task: AxTaskRef,
}

impl TaskWaker {
    /// Creates a new task waker for the given task.
    pub fn new(task: AxTaskRef) -> Self {
        Self { task }
    }

    /// Wakes up the task.
    fn wake_task(&self) {
        if self.task.state() == TaskState::Blocked {
            trace!("Waking task: {}", self.task.id());
            self.task.unblock();
        }
    }

    /// Converts this task waker into a standard waker.
    pub fn into_waker(self) -> Waker {
        use core::task::{RawWaker, RawWakerVTable};

        let task_waker = Arc::new(self);
        let ptr = Arc::into_raw(task_waker) as *const ();
        let vtable = &TaskWaker::VTABLE;
        unsafe { Waker::from_raw(RawWaker::new(ptr, vtable)) }
    }

    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        // Clone
        |ptr| {
            let arc = unsafe { Arc::from_raw(ptr as *const TaskWaker) };
            let waker = arc.clone();
            let _ = Arc::into_raw(arc);
            let ptr = Arc::into_raw(waker) as *const ();
            RawWaker::new(ptr, &TaskWaker::VTABLE)
        },
        // Wake
        |ptr| {
            let arc = unsafe { Arc::from_raw(ptr as *const TaskWaker) };
            arc.wake_task();
            drop(arc);
        },
        // Wake by reference
        |ptr| {
            let arc = unsafe { Arc::from_raw(ptr as *const TaskWaker) };
            arc.wake_task();
            let _ = Arc::into_raw(arc);
        },
        // Drop
        |ptr| {
            let arc = unsafe { Arc::from_raw(ptr as *const TaskWaker) };
            drop(arc);
        },
    );
}

/// A waker that is tied to a specific task.
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

/// Returns a waker for the current task.
pub fn current_task_waker() -> Waker {
    TaskWaker::new(axtask::current().clone()).into_waker()
}
