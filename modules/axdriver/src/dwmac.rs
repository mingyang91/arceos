use axdma::{BusAddr, DMAInfo, alloc_coherent, dealloc_coherent};
use axdriver_net::dwmac::{DwmacHal, PhysAddr as DwmacPhysAddr};
use axhal::mem::{phys_to_virt, virt_to_phys};
use core::{alloc::Layout, ptr::NonNull};

pub struct DwmacHalImpl;

impl DwmacHal for DwmacHalImpl {
    fn dma_alloc(size: usize) -> (DwmacPhysAddr, NonNull<u8>) {
        let layout = Layout::from_size_align(size, 8).unwrap();
        match unsafe { alloc_coherent(layout) } {
            Ok(dma_info) => (dma_info.bus_addr.as_u64() as usize, dma_info.cpu_addr),
            Err(_) => (0, NonNull::dangling()),
        }
    }

    unsafe fn dma_dealloc(paddr: DwmacPhysAddr, vaddr: NonNull<u8>, size: usize) -> i32 {
        let layout = Layout::from_size_align(size, 8).unwrap();
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
        virt_to_phys((vaddr.as_ptr() as usize).into()).into()
    }

    fn wait_until(duration: core::time::Duration) -> Result<(), &'static str> {
        axhal::time::busy_wait_until(duration);
        Ok(())
    }
}
 