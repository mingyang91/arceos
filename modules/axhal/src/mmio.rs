//! MMIO (Memory-Mapped I/O) interrupt handler support.

extern crate alloc;

use crate::irq::register_handler;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use kspin::SpinNoIrq;

/// Represents a Memory-Mapped I/O device address range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MmioRange {
    pub start: usize,
    pub size: usize,
}

impl MmioRange {
    /// Creates a new MMIO range.
    pub const fn new(start: usize, size: usize) -> Self {
        Self { start, size }
    }

    /// Checks if the given physical address is within this MMIO range.
    pub fn contains(&self, addr: usize) -> bool {
        addr >= self.start && addr < self.start + self.size
    }
}

// Global registry for MMIO devices
static DEVICE_REGISTRY: SpinNoIrq<BTreeMap<u32, (Option<Arc<dyn MmioDevice>>, MmioRange)>> =
    SpinNoIrq::new(BTreeMap::new());

static IRQ_NUM: AtomicU32 = AtomicU32::new(1);

// Track the current IRQ being handled
static CURRENT_IRQ: AtomicUsize = AtomicUsize::new(0);

/// Trait for devices that support MMIO interrupts.
pub trait MmioDevice: Send + Sync {
    /// Handle an MMIO interrupt.
    fn handle_interrupt(&self) -> bool;
}

// Common dispatch function
fn dispatch_irq(irq: u32) {
    let device_opt = find_device_by_irq(irq);
    if let Some(device) = device_opt {
        let handled = device.handle_interrupt();
        if !handled {
            warn!("Unhandled MMIO interrupt for device at IRQ {}", irq);
        }
    } else {
        warn!("No device registered for IRQ {}", irq);
    }
}

const S_EXT: usize = (1 << (usize::BITS - 1)) + 9;
/// Register an MMIO device and its interrupt handler.
///
/// This function maps a device to its MMIO address range, allowing
/// the system to route interrupts to the appropriate device handler.
///
/// Returns `true` if registration was successful.
pub fn register_mmio_device(range: MmioRange, device: Option<Arc<dyn MmioDevice>>) -> Option<u32> {
    let irq = IRQ_NUM.fetch_add(1, Ordering::Relaxed);
    // Register in the device registry
    let already_registered = DEVICE_REGISTRY
        .lock()
        .insert(irq, (device.clone(), range))
        .is_some();

    if already_registered {
        warn!(
            "Overwriting previously registered MMIO device at {:#x}-{:#x}",
            range.start,
            range.start + range.size
        );
    }

    // Select the appropriate static handler function
    let handler: fn() = match irq {
        0 => || dispatch_irq(0),
        1 => || dispatch_irq(1),
        2 => || dispatch_irq(2),
        3 => || dispatch_irq(3),
        4 => || dispatch_irq(4),
        5 => || dispatch_irq(5),
        6 => || dispatch_irq(6),
        7 => || dispatch_irq(7),
        8 => || dispatch_irq(8),
        _ => {
            warn!("IRQ {} not supported in this implementation", irq);
            return None;
        }
    };

    // Register the handler
    if register_handler(irq as usize, handler) {
        Some(irq)
    } else {
        None
    }
}

pub fn replace_mmio_device(irq: u32, device: Option<Arc<dyn MmioDevice>>) -> bool {
    DEVICE_REGISTRY
        .lock()
        .get_mut(&irq)
        .map(|(old_device, _)| {
            *old_device = device;
        })
        .is_some()
}

/// Find device by IRQ number.
///
/// This allows checking which device is registered to handle a specific IRQ.
pub fn find_device_by_irq(irq: u32) -> Option<Arc<dyn MmioDevice>> {
    DEVICE_REGISTRY
        .lock()
        .get(&irq)
        .map(|(device, _)| device.clone())
        .flatten()
}

/// Dumps information about all registered MMIO devices.
///
/// Useful for debugging interrupt routing issues.
pub fn dump_mmio_registry() {
    info!("MMIO Device Registry:");
    for (irq, (_, range)) in DEVICE_REGISTRY.lock().iter() {
        info!(
            "  {:#x}-{:#x} => IRQ {}",
            range.start,
            range.start + range.size,
            irq
        );
    }
}
