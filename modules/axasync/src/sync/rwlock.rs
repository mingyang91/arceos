//! Async read-write lock implementation.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::cell::UnsafeCell;
use core::fmt;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};
use spin::Mutex as SpinMutex;

// Constants for the state field in RwLockInner
const WRITER: usize = !0;
const READER_MASK: usize = WRITER - 1;

/// An asynchronous reader-writer lock.
///
/// This type of lock allows multiple readers or a single writer at any point in time.
/// The write lock has priority over the read lock to prevent reader starvation.
pub struct RwLock<T: ?Sized> {
    inner: Arc<RwLockInner<T>>,
}

struct RwLockInner<T: ?Sized> {
    // The actual data being protected
    data: Box<UnsafeCell<T>>,
    // State of the lock:
    // - If state == WRITER, the lock is exclusively (write) locked.
    // - If state == 0, the lock is unlocked.
    // - If state & READER_MASK > 0, the lock is shared (read) locked by state readers.
    state: AtomicUsize,
    // Waiting writers
    write_waiters: SpinMutex<VecDeque<Waker>>,
    // Waiting readers
    read_waiters: SpinMutex<VecDeque<Waker>>,
}

unsafe impl<T: ?Sized + Send + Sync> Send for RwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    /// Creates a new async read-write lock.
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(RwLockInner {
                data: Box::new(UnsafeCell::new(data)),
                state: AtomicUsize::new(0),
                write_waiters: SpinMutex::new(VecDeque::new()),
                read_waiters: SpinMutex::new(VecDeque::new()),
            }),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Attempts to acquire this lock with shared read access.
    ///
    /// If the lock could not be acquired because it is currently held exclusively,
    /// then `None` is returned. Otherwise, a guard is returned which will release
    /// the shared access when dropped.
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        let state = self.inner.state.load(Ordering::Acquire);
        if state == WRITER {
            return None;
        }

        let new_state = state.checked_add(1).expect("Too many readers");
        if self
            .inner
            .state
            .compare_exchange(state, new_state, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            Some(RwLockReadGuard {
                lock: self,
                inner: self.inner.clone(),
            })
        } else {
            None
        }
    }

    /// Attempts to lock this rwlock with exclusive write access.
    ///
    /// If the lock could not be acquired at this time, then `None` is returned.
    /// Otherwise, an RAII guard is returned which will release the lock when
    /// dropped.
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        if self
            .inner
            .state
            .compare_exchange(0, WRITER, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
        {
            Some(RwLockWriteGuard {
                lock: self,
                inner: self.inner.clone(),
            })
        } else {
            None
        }
    }

    /// Locks this rwlock with shared read access.
    ///
    /// Returns a future that resolves to a guard when the read lock is acquired.
    pub fn read(&self) -> RwLockReadFuture<'_, T> {
        RwLockReadFuture {
            lock: self,
            inner: self.inner.clone(),
        }
    }

    /// Locks this rwlock with exclusive write access.
    ///
    /// Returns a future that resolves to a guard when the write lock is acquired.
    pub fn write(&self) -> RwLockWriteFuture<'_, T> {
        RwLockWriteFuture {
            lock: self,
            inner: self.inner.clone(),
        }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for RwLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("RwLock");
        match self.try_read() {
            Some(guard) => d.field("data", &&*guard),
            None => d.field("data", &"<locked>"),
        }
        .finish()
    }
}

impl<T: ?Sized + Default> Default for RwLock<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> From<T> for RwLock<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

/// A future that resolves when the read lock is acquired.
pub struct RwLockReadFuture<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
    inner: Arc<RwLockInner<T>>,
}

impl<'a, T: ?Sized> Future for RwLockReadFuture<'a, T> {
    type Output = RwLockReadGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Fast path: try to acquire the read lock
        if let Some(guard) = self.lock.try_read() {
            return Poll::Ready(guard);
        }

        // Add our waker to the list of waiters
        self.inner.read_waiters.lock().push_back(cx.waker().clone());

        // Try again in case the lock was released between when we last checked
        // and when we added our waker to the waiters list
        if let Some(guard) = self.lock.try_read() {
            // We successfully got the lock, so we won't be woken up by another task
            // Remove our waker from the queue to avoid a spurious wake-up
            let _ = self
                .inner
                .read_waiters
                .lock()
                .iter()
                .position(|w| w.will_wake(cx.waker()))
                .map(|pos| self.inner.read_waiters.lock().remove(pos));

            Poll::Ready(guard)
        } else {
            Poll::Pending
        }
    }
}

/// A future that resolves when the write lock is acquired.
pub struct RwLockWriteFuture<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
    inner: Arc<RwLockInner<T>>,
}

impl<'a, T: ?Sized> Future for RwLockWriteFuture<'a, T> {
    type Output = RwLockWriteGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Fast path: try to acquire the write lock
        if let Some(guard) = self.lock.try_write() {
            return Poll::Ready(guard);
        }

        // Add our waker to the list of writer waiters
        self.inner
            .write_waiters
            .lock()
            .push_back(cx.waker().clone());

        // Try again in case the lock was released between when we last checked
        // and when we added our waker to the waiters list
        if let Some(guard) = self.lock.try_write() {
            // We successfully got the lock, so we won't be woken up by another task
            // Remove our waker from the queue to avoid a spurious wake-up
            let _ = self
                .inner
                .write_waiters
                .lock()
                .iter()
                .position(|w| w.will_wake(cx.waker()))
                .map(|pos| self.inner.write_waiters.lock().remove(pos));

            Poll::Ready(guard)
        } else {
            Poll::Pending
        }
    }
}

/// A guard that provides shared read access to the protected data.
pub struct RwLockReadGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
    inner: Arc<RwLockInner<T>>,
}

impl<'a, T: ?Sized> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        // Decrement the read count
        let prev = self.inner.state.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(prev != 0 && prev != WRITER, "Invalid RwLock state");

        // If this was the last reader and there are waiting writers, wake one up
        if prev == 1 {
            if let Some(waker) = self.inner.write_waiters.lock().pop_front() {
                waker.wake();
            }
        }
    }
}

impl<'a, T: ?Sized> Deref for RwLockReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: We know we have shared read access as long as the guard exists
        unsafe { &*self.inner.data.get() }
    }
}

impl<'a, T: ?Sized + fmt::Debug> fmt::Debug for RwLockReadGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for RwLockReadGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

/// A guard that provides exclusive write access to the protected data.
pub struct RwLockWriteGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
    inner: Arc<RwLockInner<T>>,
}

impl<'a, T: ?Sized> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        // Release the write lock
        let old = self.inner.state.swap(0, Ordering::AcqRel);
        debug_assert_eq!(old, WRITER, "Invalid RwLock state");

        // Prefer writers over readers to prevent writer starvation
        if let Some(waker) = self.inner.write_waiters.lock().pop_front() {
            // Wake up a waiting writer
            waker.wake();
        } else {
            // Wake up all waiting readers
            let mut readers = self.inner.read_waiters.lock();
            for waker in readers.drain(..) {
                waker.wake();
            }
        }
    }
}

impl<'a, T: ?Sized> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: We know we have exclusive access as long as the guard exists
        unsafe { &*self.inner.data.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: We know we have exclusive access as long as the guard exists
        unsafe { &mut *self.inner.data.get() }
    }
}

impl<'a, T: ?Sized + fmt::Debug> fmt::Debug for RwLockWriteGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for RwLockWriteGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
