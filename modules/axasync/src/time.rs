//! Async time-related functions.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use core::time::Duration;

use axhal::time::{TimeValue, monotonic_time as current_time};

/// A future that completes after a specified duration of time.
pub struct Sleep {
    deadline: TimeValue,
    registered_waker: Option<Waker>,
}

impl Sleep {
    /// Creates a new future that completes after the specified duration.
    pub fn new(duration: Duration) -> Self {
        let deadline = current_time() + duration;
        debug!("Sleeping until {:?}", deadline);
        Self::until(deadline)
    }

    /// Creates a new future that completes at the specified deadline.
    pub fn until(deadline: TimeValue) -> Self {
        Self {
            deadline,
            registered_waker: None,
        }
    }

    /// Returns the instant at which this sleep will complete.
    pub fn deadline(&self) -> TimeValue {
        self.deadline
    }

    /// Resets the sleep to complete after the specified duration.
    pub fn reset(&mut self, duration: Duration) {
        self.deadline = current_time() + duration;
    }

    /// Resets the sleep to complete at the specified deadline.
    pub fn reset_until(&mut self, deadline: TimeValue) {
        self.deadline = deadline;
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        trace!("sleep poll");
        let now = current_time();
        if now >= self.deadline {
            Poll::Ready(())
        } else {
            #[cfg(feature = "timer")]
            {
                let mut this = self.get_mut();
                if let Some(ref waker) = this.registered_waker {
                    if !waker.will_wake(cx.waker()) {
                        this.registered_waker = Some(cx.waker().clone());
                        crate::waker::wake_at(this.deadline, cx.waker().clone());
                    }
                } else {
                    this.registered_waker = Some(cx.waker().clone());
                    crate::waker::wake_at(this.deadline, cx.waker().clone());
                }
            }
            // info!("Sleeping for {:?}", self.deadline - now);
            Poll::Pending
        }
    }
}

/// Async version of [`axtask::sleep`], that sleeps for the specified duration.
pub async fn sleep(duration: Duration) {
    Sleep::new(duration).await
}

/// Async version of [`axtask::sleep_until`], that sleeps until the specified deadline.
pub async fn sleep_until(deadline: TimeValue) {
    Sleep::until(deadline).await
}

/// Extension trait that adds timeout methods to futures.
pub trait TimeoutExt: Future {
    /// Creates a new future that times out after the specified duration.
    fn timeout(self, duration: Duration) -> Timeout<Self>
    where
        Self: Sized,
    {
        Timeout::new(self, duration)
    }

    /// Creates a new future that times out at the specified deadline.
    fn timeout_at(self, deadline: TimeValue) -> Timeout<Self>
    where
        Self: Sized,
    {
        Timeout::until(self, deadline)
    }
}

impl<F: Future> TimeoutExt for F {}

/// A future that completes when either the inner future completes or when the
/// timeout is reached, whichever comes first.
pub struct Timeout<F: Future> {
    future: F,
    sleep: Sleep,
}

impl<F: Future> Timeout<F> {
    /// Creates a new timeout future with the specified duration.
    pub fn new(future: F, duration: Duration) -> Self {
        Self {
            future,
            sleep: Sleep::new(duration),
        }
    }

    /// Creates a new timeout future with the specified deadline.
    pub fn until(future: F, deadline: TimeValue) -> Self {
        Self {
            future,
            sleep: Sleep::until(deadline),
        }
    }
}

/// Error returned by the [`Timeout`] future if the inner future times out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutError;

impl<F: Future> Future for Timeout<F> {
    type Output = Result<F::Output, TimeoutError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        trace!("timeout poll");
        // Safety: We're not moving any fields out of the pinned future
        let this = unsafe { self.get_unchecked_mut() };

        // First, try polling the inner future
        let future = unsafe { Pin::new_unchecked(&mut this.future) };
        if let Poll::Ready(result) = future.poll(cx) {
            return Poll::Ready(Ok(result));
        }

        // Then check if the timeout has been reached
        let sleep = unsafe { Pin::new_unchecked(&mut this.sleep) };
        if let Poll::Ready(()) = sleep.poll(cx) {
            return Poll::Ready(Err(TimeoutError));
        }

        Poll::Pending
    }
}
