#[allow(unused_imports)]
use crate::{AllDevices, prelude::*};
use axhal::mmio::MmioRange;
impl AllDevices {
    pub(crate) fn probe_bus_devices(&mut self) {
        warn!("probing bus devices...");
        // TODO: parse device tree
        #[cfg(feature = "virtio")]
        for reg in axconfig::devices::VIRTIO_MMIO_REGIONS {
            for_each_drivers!(type Driver, {
                if let Some(dev) = Driver::probe_mmio(reg.0, reg.1) {
                    info!(
                        "registered a new {:?} device at [PA:{:#x}, PA:{:#x}): {:?}",
                        dev.device_type(),
                        reg.0, reg.0 + reg.1,
                        dev.device_name(),
                    );
                    let irq = axhal::mmio::register_mmio_device(MmioRange::new(reg.0, reg.1), None)
                        .expect("failed to register MMIO device");
                    self.add_device(dev, irq);
                    continue; // skip to the next device
                }
            });
        }
    }
}
