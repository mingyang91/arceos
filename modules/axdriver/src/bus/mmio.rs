#[allow(unused_imports)]
use crate::{AllDevices, AxDeviceEnum, prelude::*};

impl AllDevices {
    pub(crate) fn probe_bus_devices(&mut self) {
        info!("probing bus devices...");
        // Probe regular MMIO devices
        for reg in axconfig::devices::MMIO_REGIONS {
            if reg.0 == 0x1304_0000 {
                info!("skipping GPIO MMIO region");
                continue;
            }
            for_each_drivers!(type Driver, {
                if let Some(dev) = Driver::probe_mmio(reg.0, reg.1) {
                    // TODO: hardcode for tutorial
                    if reg.0 == 0x16030000 {
                        info!(
                            "registered a new {:?} device at [PA:{:#x}, PA:{:#x}): {:?}",
                            dev.device_type(),
                            reg.0, reg.0 + reg.1,
                            dev.device_name(),
                        );
                        self.add_device(dev, 7); // GMAC0 IRQ
                    } else if reg.0 == 0x16040000 {
                        info!(
                            "registered a new {:?} device at [PA:{:#x}, PA:{:#x}): {:?}",
                            dev.device_type(),
                            reg.0, reg.0 + reg.1,
                            dev.device_name(),
                        );
                        self.add_device(dev, 78); // GMAC1 IRQ
                    } else {
                        unimplemented!("unknown device");
                    }

                    continue; // skip to the next device
                }
            });
        }

        let mut irq = 0;
        // TODO: parse device tree
        #[cfg(feature = "virtio")]
        for reg in axconfig::devices::VIRTIO_MMIO_REGIONS {
            irq += 1;
            for_each_drivers!(type Driver, {
                if let Some(dev) = Driver::probe_mmio(reg.0, reg.1) {
                    info!(
                        "registered a new {:?} device at [PA:{:#x}, PA:{:#x}): {:?}",
                        dev.device_type(),
                        reg.0, reg.0 + reg.1,
                        dev.device_name(),
                    );

                    self.add_device(dev, irq);
                    continue; // skip to the next device
                }
            });
        }
    }
}
