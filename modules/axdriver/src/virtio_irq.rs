//! Interrupt support for VirtIO devices

use crate::virtio::VirtIoHalImpl;
use axdriver_virtio::{MmioTransport, VirtIoBlkDev, VirtIoNetDev};
use axhal::irq::{self, IrqHandler};
use axhal::platform::riscv64_qemu_virt::irq::{VIRTIO_BLK_IRQ, VIRTIO_NET_IRQ};
use kspin::SpinNoIrq;
use lazyinit::LazyInit;

/// Global reference to the virtio-net device for interrupt handling
static VIRTIO_NET_DEV: LazyInit<SpinNoIrq<VirtIoNetDev<VirtIoHalImpl, MmioTransport, 64>>> =
    LazyInit::new();

/// Global reference to the virtio-blk device for interrupt handling
static VIRTIO_BLK_DEV: LazyInit<SpinNoIrq<VirtIoBlkDev<VirtIoHalImpl, MmioTransport>>> =
    LazyInit::new();

/// Initialize interrupt handling for virtio-net device
pub fn init_virtio_net_irq(dev: VirtIoNetDev<VirtIoHalImpl, MmioTransport, 64>) {
    // Store device in global static for IRQ handler access
    VIRTIO_NET_DEV.init_once(SpinNoIrq::new(dev));

    // Register IRQ handler
    irq::register_handler(VIRTIO_NET_IRQ, virtio_net_irq_handler);
}

/// Initialize interrupt handling for virtio-blk device
pub fn init_virtio_blk_irq(dev: VirtIoBlkDev<VirtIoHalImpl, MmioTransport>) {
    // Store device in global static for IRQ handler access
    VIRTIO_BLK_DEV.init_once(SpinNoIrq::new(dev));

    // Register IRQ handler
    irq::register_handler(VIRTIO_BLK_IRQ, virtio_blk_irq_handler);
}

/// Virtio network device interrupt handler
fn virtio_net_irq_handler() {
    if VIRTIO_NET_DEV.is_inited() {
        let mut dev = VIRTIO_NET_DEV.lock();

        // Acknowledge the interrupt
        dev.ack_interrupt();

        // Process any pending packets
        // In a real implementation, this would signal any waiting tasks
        trace!("Virtio-net interrupt received");
    }
}

/// Virtio block device interrupt handler
fn virtio_blk_irq_handler() {
    if VIRTIO_BLK_DEV.is_inited() {
        let mut dev = VIRTIO_BLK_DEV.lock();

        // Acknowledge the interrupt
        dev.ack_interrupt();

        // Process any pending requests
        // In a real implementation, this would signal any waiting tasks
        trace!("Virtio-blk interrupt received");
    }
}
