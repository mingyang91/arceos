//! Async MMIO (Memory-Mapped I/O) operations.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};
use kspin::SpinNoIrq;

use crate::waker::SimpleWaker;

/// Type for MMIO device event ID
pub type MmioEventId = u64;

static MMIO_EVENT_COUNTER: AtomicU64 = AtomicU64::new(1);

/// The handler for an MMIO device event.
///
/// This trait must be implemented by device drivers that want to support async operations
/// over MMIO interrupts.
pub trait MmioEventHandler: Send + Sync {
    /// The type of data passed to the event handler when the MMIO event is triggered.
    type Data: Send + Sync + Clone + 'static;

    /// Register an MMIO event.
    ///
    /// This method should register the device's IRQ and provide a way to notify
    /// when the event occurs.
    fn register_event(&self, event_id: MmioEventId, waker: Waker) -> bool;

    /// Cancel a previously registered MMIO event.
    fn cancel_event(&self, event_id: MmioEventId) -> bool;
}

/// A future that waits for an MMIO event to occur.
pub struct MmioEvent<H: MmioEventHandler> {
    event_handler: Arc<H>,
    event_id: MmioEventId,
    registered: bool,
    completed: bool,
}

impl<H: MmioEventHandler> MmioEvent<H> {
    /// Creates a new MMIO event future.
    pub fn new(event_handler: Arc<H>) -> Self {
        let event_id = MMIO_EVENT_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self {
            event_handler,
            event_id,
            registered: false,
            completed: false,
        }
    }

    /// Get the event ID for this MMIO event.
    pub fn event_id(&self) -> MmioEventId {
        self.event_id
    }
}

impl<H: MmioEventHandler> Future for MmioEvent<H> {
    type Output = MmioEventId;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            return Poll::Ready(self.event_id);
        }

        if !self.registered {
            let registered = self
                .event_handler
                .register_event(self.event_id, cx.waker().clone());
            if !registered {
                // If registration failed, we consider the event as completed
                // This prevents infinite polling loop
                warn!("Failed to register MMIO event: {}", self.event_id);
                self.completed = true;
                return Poll::Ready(self.event_id);
            }
            self.registered = true;
        }

        Poll::Pending
    }
}

impl<H: MmioEventHandler> Drop for MmioEvent<H> {
    fn drop(&mut self) {
        if self.registered && !self.completed {
            self.event_handler.cancel_event(self.event_id);
        }
    }
}

/// Utility struct to manage a collection of MMIO wakers.
///
/// This provides a convenient way for device drivers to manage multiple wakers
/// for different types of events.
pub struct MmioWakerSet {
    wakers: SpinNoIrq<alloc::collections::BTreeMap<MmioEventId, Waker>>,
}

impl MmioWakerSet {
    /// Creates a new empty waker set.
    pub fn new() -> Self {
        Self {
            wakers: SpinNoIrq::new(alloc::collections::BTreeMap::new()),
        }
    }

    /// Registers a waker for the given event ID.
    ///
    /// Returns true if registration was successful.
    pub fn register(&self, event_id: MmioEventId, waker: Waker) -> bool {
        let mut wakers = self.wakers.lock();
        wakers.insert(event_id, waker);
        true
    }

    /// Removes a previously registered waker.
    ///
    /// Returns true if cancellation was successful.
    pub fn cancel(&self, event_id: MmioEventId) -> bool {
        let mut wakers = self.wakers.lock();
        wakers.remove(&event_id).is_some()
    }

    /// Wake all wakers that match the given predicate.
    ///
    /// The predicate takes an event ID and should return true if the event
    /// should be woken.
    pub fn wake_matching<F>(&self, predicate: F)
    where
        F: Fn(MmioEventId) -> bool,
    {
        let mut wakers_to_wake = Vec::new();
        let mut event_ids_to_remove = Vec::new();

        // First, collect all wakers that match the predicate
        {
            let wakers = self.wakers.lock();
            for (&event_id, waker) in wakers.iter() {
                if predicate(event_id) {
                    wakers_to_wake.push(waker.clone());
                    event_ids_to_remove.push(event_id);
                }
            }
        }

        // Then, remove the matched event IDs
        {
            let mut wakers = self.wakers.lock();
            for event_id in event_ids_to_remove {
                wakers.remove(&event_id);
            }
        }

        // Finally, wake all collected wakers
        for waker in wakers_to_wake {
            waker.wake();
        }
    }

    /// Wake a specific event.
    ///
    /// Returns true if the event was found and woken.
    pub fn wake_event(&self, event_id: MmioEventId) -> bool {
        let waker = {
            let mut wakers = self.wakers.lock();
            wakers.remove(&event_id)
        };

        if let Some(waker) = waker {
            waker.wake();
            true
        } else {
            false
        }
    }
}
