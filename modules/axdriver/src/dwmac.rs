use axdma::{BusAddr, DMAInfo, alloc_coherent, dealloc_coherent};
use axdriver_net::dwmac::{DwmacHal, PhysAddr as DwmacPhysAddr};
use axhal::mem::{phys_to_virt, virt_to_phys};
use core::{alloc::Layout, ptr::NonNull};

pub struct DwmacHalImpl;

impl DwmacHal for DwmacHalImpl {
    fn dma_alloc(size: usize) -> (DwmacPhysAddr, NonNull<u8>) {
        let layout = Layout::from_size_align(size, 16).unwrap(); // 16-byte alignment for descriptors
        match unsafe { alloc_coherent(layout) } {
            Ok(dma_info) => {
                log::debug!(
                    "DMA alloc: size={}, cpu_addr={:p}, bus_addr={:#x}",
                    size,
                    dma_info.cpu_addr.as_ptr(),
                    dma_info.bus_addr.as_u64()
                );
                // Return bus address for hardware and CPU virtual address
                (dma_info.bus_addr.as_u64() as usize, dma_info.cpu_addr)
            }
            Err(e) => {
                log::error!("DMA allocation failed: {:?}", e);
                (0, NonNull::dangling())
            }
        }
    }

    unsafe fn dma_dealloc(paddr: DwmacPhysAddr, vaddr: NonNull<u8>, size: usize) -> i32 {
        let layout = Layout::from_size_align(size, 16).unwrap();
        let dma_info = DMAInfo {
            cpu_addr: vaddr,
            bus_addr: BusAddr::from(paddr as u64),
        };
        unsafe { dealloc_coherent(dma_info, layout) };
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: DwmacPhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(phys_to_virt(paddr.into()).as_mut_ptr()).unwrap()
    }

    unsafe fn mmio_virt_to_phys(vaddr: NonNull<u8>, _size: usize) -> DwmacPhysAddr {
        // For DMA buffers allocated through dma_alloc, we need to convert back to bus address
        // This is a simplified approach - in a real implementation, we'd track the mapping
        let virt_addr = vaddr.as_ptr() as usize;
        let phys_addr = virt_to_phys(virt_addr.into());

        // Convert physical address to bus address using the platform offset
        let bus_addr = phys_addr.as_usize() + axconfig::plat::PHYS_BUS_OFFSET;
        log::trace!(
            "virt_to_phys: virt={:#x} -> phys={:#x} -> bus={:#x}",
            virt_addr,
            phys_addr.as_usize(),
            bus_addr
        );
        bus_addr
    }

    fn wait_until(duration: core::time::Duration) -> Result<(), &'static str> {
        axhal::time::busy_wait_until(duration);
        Ok(())
    }
}
