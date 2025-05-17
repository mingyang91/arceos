//! Async mutex implementation.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::cell::UnsafeCell;
use core::fmt;
use core::future::Future;
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll, Waker};
use spin::Mutex as SpinMutex;

/// An asynchronous mutual exclusion primitive useful for protecting shared data.
///
/// This mutex will wait asynchronously if the lock cannot be acquired immediately.
/// This means that the executor can make progress with other tasks while waiting for the lock.
pub struct Mutex<T: ?Sized> {
    // The inner state shared between all lock holders
    inner: Arc<MutexInner<T>>,
}

struct MutexInner<T: ?Sized> {
    // The actual data being protected
    data: Box<UnsafeCell<T>>,
    // Whether the mutex is locked
    locked: AtomicBool,
    // Queue of waiters
    waiters: SpinMutex<VecDeque<Waker>>,
}

impl<T> Mutex<T> {
    /// Creates a new async mutex.
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(MutexInner {
                data: Box::new(UnsafeCell::new(data)),
                locked: AtomicBool::new(false),
                waiters: SpinMutex::new(VecDeque::new()),
            }),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Attempts to acquire this lock immediately.
    ///
    /// If the lock could not be acquired at this time, then `None` is returned.
    /// Otherwise, an RAII guard is returned which will release the lock when
    /// dropped.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if !self.inner.locked.swap(true, Ordering::Acquire) {
            Some(MutexGuard {
                mutex: self,
                inner: self.inner.clone(),
            })
        } else {
            None
        }
    }

    /// Acquires this lock asynchronously.
    ///
    /// This function will return a future that will resolve once the lock
    /// has been successfully acquired.
    pub fn lock(&self) -> MutexLockFuture<'_, T> {
        MutexLockFuture {
            mutex: self,
            inner: self.inner.clone(),
        }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Mutex");
        match self.try_lock() {
            Some(guard) => d.field("data", &&*guard),
            None => d.field("data", &"<locked>"),
        }
        .finish()
    }
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

impl<T: ?Sized + Default> Default for Mutex<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

/// A future that resolves when the mutex is acquired.
pub struct MutexLockFuture<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
    inner: Arc<MutexInner<T>>,
}

impl<'a, T: ?Sized> Future for MutexLockFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        trace!("mutex lock poll");
        // Fast path: try to acquire the lock without going to sleep
        if let Some(guard) = self.mutex.try_lock() {
            return Poll::Ready(guard);
        }

        // Add our waker to the list of waiters
        self.inner.waiters.lock().push_back(cx.waker().clone());

        // Try again in case the mutex was unlocked between when we last checked
        // and when we added our waker to the waiters list
        if let Some(guard) = self.mutex.try_lock() {
            // We successfully got the lock, so we won't be woken up by another task
            // Remove our waker from the queue to avoid a spurious wake-up
            let _ = self
                .inner
                .waiters
                .lock()
                .iter()
                .position(|w| w.will_wake(cx.waker()))
                .map(|pos| self.inner.waiters.lock().remove(pos));

            Poll::Ready(guard)
        } else {
            Poll::Pending
        }
    }
}

/// An RAII guard that releases the mutex when dropped.
pub struct MutexGuard<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
    inner: Arc<MutexInner<T>>,
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        // Release the lock
        self.inner.locked.store(false, Ordering::Release);

        // Wake up a waiter if there is one
        if let Some(waker) = self.inner.waiters.lock().pop_front() {
            waker.wake();
        }
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // Safety: We know that we have exclusive access to the data
        // as long as the guard exists.
        unsafe { &*self.inner.data.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: We know that we have exclusive access to the data
        // as long as the guard exists.
        unsafe { &mut *self.inner.data.get() }
    }
}

impl<'a, T: ?Sized + fmt::Debug> fmt::Debug for MutexGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for MutexGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
