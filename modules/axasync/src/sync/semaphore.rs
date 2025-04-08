//! Async semaphore implementation.

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::fmt;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};
use spin::Mutex as SpinMutex;

/// An asynchronous semaphore.
///
/// This type of semaphore can be used to restrict access to a resource
/// to a fixed number of concurrent accessors.
pub struct Semaphore {
    inner: Arc<SemaphoreInner>,
}

#[derive(Debug)]
struct SemaphoreInner {
    // Current number of available permits
    permits: AtomicUsize,
    // Maximum number of permits
    max_permits: usize,
    // Queue of tasks waiting for permits
    waiters: SpinMutex<VecDeque<Waker>>,
}

impl Semaphore {
    /// Creates a new semaphore with the given number of permits.
    pub fn new(permits: usize) -> Self {
        Self {
            inner: Arc::new(SemaphoreInner {
                permits: AtomicUsize::new(permits),
                max_permits: permits,
                waiters: SpinMutex::new(VecDeque::new()),
            }),
        }
    }

    /// Returns the current number of available permits.
    pub fn available_permits(&self) -> usize {
        self.inner.permits.load(Ordering::Acquire)
    }

    /// Returns the maximum number of permits.
    pub fn max_permits(&self) -> usize {
        self.inner.max_permits
    }

    /// Attempts to acquire a permit from the semaphore.
    ///
    /// If no permits are available, returns `None`.
    /// Otherwise, returns a guard that will release the permit when dropped.
    pub fn try_acquire(&self) -> Option<SemaphorePermit> {
        let permits = self.inner.permits.fetch_sub(1, Ordering::AcqRel);
        if permits > 0 {
            Some(SemaphorePermit {
                inner: self.inner.clone(),
            })
        } else {
            // Restore the permit count
            self.inner.permits.fetch_add(1, Ordering::Release);
            None
        }
    }

    /// Acquires a permit from the semaphore asynchronously.
    ///
    /// Returns a future that resolves to a guard when a permit is acquired.
    pub fn acquire(&self) -> SemaphoreAcquireFuture {
        SemaphoreAcquireFuture {
            semaphore: self,
            inner: self.inner.clone(),
        }
    }
}

impl fmt::Debug for Semaphore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Semaphore")
            .field("permits", &self.available_permits())
            .field("max_permits", &self.max_permits())
            .finish()
    }
}

impl Clone for Semaphore {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// A future that resolves when a permit is acquired from the semaphore.
pub struct SemaphoreAcquireFuture<'a> {
    semaphore: &'a Semaphore,
    inner: Arc<SemaphoreInner>,
}

impl<'a> Future for SemaphoreAcquireFuture<'a> {
    type Output = SemaphorePermit;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Fast path: try to acquire a permit immediately
        if let Some(permit) = self.semaphore.try_acquire() {
            return Poll::Ready(permit);
        }

        // Add our waker to the queue
        self.inner.waiters.lock().push_back(cx.waker().clone());

        // Try again in case a permit was released between when we last checked
        // and when we added our waker to the queue
        if let Some(permit) = self.semaphore.try_acquire() {
            // We successfully got a permit, so we won't be woken up by another task
            // Remove our waker from the queue to avoid a spurious wake-up
            let _ = self
                .inner
                .waiters
                .lock()
                .iter()
                .position(|w| w.will_wake(cx.waker()))
                .map(|pos| self.inner.waiters.lock().remove(pos));

            Poll::Ready(permit)
        } else {
            Poll::Pending
        }
    }
}

/// A permit from the semaphore.
///
/// This guard automatically releases the permit when dropped.
#[derive(Debug)]
pub struct SemaphorePermit {
    inner: Arc<SemaphoreInner>,
}

impl Drop for SemaphorePermit {
    fn drop(&mut self) {
        // Release the permit
        let permits = self.inner.permits.fetch_add(1, Ordering::AcqRel);
        debug_assert!(
            permits < self.inner.max_permits,
            "Semaphore permit count error"
        );

        // Wake up a waiting task if there are any
        if let Some(waker) = self.inner.waiters.lock().pop_front() {
            waker.wake();
        }
    }
}

/// A specialized semaphore that only allows a single permit.
/// This can be used as a "mutex" that can be locked and unlocked
/// across different tasks, unlike a regular mutex.
#[derive(Debug, Clone)]
pub struct Barrier {
    semaphore: Semaphore,
}

impl Barrier {
    /// Creates a new barrier in the given state.
    pub fn new(locked: bool) -> Self {
        Self {
            semaphore: Semaphore::new(if locked { 0 } else { 1 }),
        }
    }

    /// Returns a future that resolves when the barrier is acquired.
    pub async fn acquire(&self) -> BarrierGuard {
        let permit = self.semaphore.acquire().await;
        BarrierGuard {
            barrier: self.clone(),
            _permit: permit,
        }
    }

    /// Attempts to acquire the barrier immediately.
    pub fn try_acquire(&self) -> Option<BarrierGuard> {
        self.semaphore.try_acquire().map(|permit| BarrierGuard {
            barrier: self.clone(),
            _permit: permit,
        })
    }

    /// Checks if the barrier is currently released.
    pub fn is_released(&self) -> bool {
        self.semaphore.available_permits() > 0
    }
}

/// A guard that releases the barrier when dropped.
#[derive(Debug)]
pub struct BarrierGuard {
    barrier: Barrier,
    _permit: SemaphorePermit,
}

impl BarrierGuard {
    /// Explicitly releases the barrier before the guard is dropped.
    pub fn release(self) {
        // The permit will be released when self is dropped
        drop(self);
    }
}
