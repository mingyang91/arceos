//! Interrupt support for VirtIO devices

use axhal::irq::{self, IrqHandler};

#[cfg(feature = "net")]
mod net {
    use crate::virtio::VirtIoHalImpl;
    use axdriver_virtio::{MmioTransport, VirtIoNetDev};
    use kspin::SpinNoIrq;
    use lazyinit::LazyInit;

    const VIRTIO_NET_IRQ: usize = 2;

    /// Initialize interrupt handling for virtio-net device
    pub fn init_virtio_net_irq() {
        // Register IRQ handler
        for irq in 1..=16 {
            if axhal::irq::register_handler(irq, virtio_net_irq_handler) {
                info!("Registered virtio-net IRQ handler");
            } else {
                warn!("Failed to register virtio-net IRQ handler");
            }
        }
    }

    /// Virtio network device interrupt handler
    fn virtio_net_irq_handler() {
        error!("Virtio-net interrupt received");
    }
}

#[cfg(feature = "block")]
mod blk {
    use crate::virtio::VirtIoHalImpl;
    use axdriver_virtio::{MmioTransport, VirtIoBlkDev};
    use kspin::SpinNoIrq;
    use lazyinit::LazyInit;

    const VIRTIO_BLK_IRQ: usize = 3;

    /// Initialize interrupt handling for virtio-blk device
    pub fn init_virtio_blk_irq() {
        // Register IRQ handler
        axhal::irq::register_handler(VIRTIO_BLK_IRQ, virtio_blk_irq_handler);
    }

    /// Virtio block device interrupt handler
    fn virtio_blk_irq_handler() {
        trace!("Virtio-blk interrupt received");
    }
}

#[cfg(feature = "net")]
pub use net::*;

#[cfg(feature = "block")]
pub use blk::*;
