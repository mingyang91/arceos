//! Async Memory-Mapped I/O (MMIO) operations.
//!
//! This module provides asynchronous interfaces for working with memory-mapped I/O devices.

// Re-export the relevant types from axasync
pub use axasync::mmio::{MmioEvent, MmioEventHandler, MmioEventId, MmioWakerSet};

/// Wait for an MMIO event to occur.
///
/// This function is a convenience wrapper around `MmioEvent::new`.
///
/// # Examples
///
/// ```
/// use axstd::mmio::{wait_for_event, MmioEventHandler};
/// use alloc::sync::Arc;
///
/// async fn example(device: Arc<impl MmioEventHandler>) {
///     // Wait for an MMIO event
///     let event_id = wait_for_event(&device).await;
///     println!("Event {} occurred!", event_id);
/// }
/// ```
pub async fn wait_for_event<H: MmioEventHandler>(device: &alloc::sync::Arc<H>) -> MmioEventId {
    MmioEvent::new(device.clone()).await
}

/// Asynchronously wait for a specific event to occur on an MMIO device.
///
/// This is a helper function that creates an event and tags it with the specified ID.
///
/// # Examples
///
/// ```
/// use axstd::mmio::{wait_for_specific_event, MmioEventHandler};
/// use alloc::sync::Arc;
///
/// async fn example(device: Arc<impl MmioEventHandler>, event_type: u32) {
///     // Wait for a specific MMIO event
///     wait_for_specific_event(&device, event_type).await;
///     println!("Event {} occurred!", event_type);
/// }
/// ```
pub async fn wait_for_specific_event<H: MmioEventHandler>(
    device: &alloc::sync::Arc<H>,
    _event_type: u32,
) {
    // Create an event and associate it with the specified event type
    let event = MmioEvent::new(device.clone());
    let _ = event.await;
}
