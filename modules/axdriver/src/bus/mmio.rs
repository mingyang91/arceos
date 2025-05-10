#[allow(unused_imports)]
use crate::{AllDevices, AxDeviceEnum, prelude::*};

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
                    // let irq = axhal::mmio::register_mmio_device(MmioRange::new(reg.0, reg.1), None)
                    //     .expect("failed to register MMIO device");

                    // Register interrupt handlers for virtio devices
                    #[cfg(feature = "irq")]
                    self.setup_device_irq(&dev);

                    self.add_device(dev);
                    continue; // skip to the next device
                }
            });
        }
    }

    #[cfg(all(feature = "virtio", feature = "irq"))]
    fn setup_device_irq(&mut self, dev: &AxDeviceEnum) {
        use crate::virtio_irq;
        use axdriver_base::DeviceType;

        match dev.device_type() {
            DeviceType::Net => {
                // TODO: Actually implement passing ownership of device to IRQ handler
                // This is just a placeholder for design purposes
                virtio_irq::init_virtio_net_irq();
                info!("Registered virtio-net IRQ handler");
            }
            // DeviceType::Block => {
            //     // TODO: Actually implement passing ownership of device to IRQ handler
            //     // This is just a placeholder for design purposes
            //     #[]
            //     virtio_irq::init_virtio_blk_irq();
            //     info!("Registered virtio-blk IRQ handler");
            // }
            _ => {}
        }
    }
}
