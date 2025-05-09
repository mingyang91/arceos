//! Platform-Level Interrupt Controller (PLIC) driver for RISC-V

use crate::mem::{PhysAddr, phys_to_virt};
use core::ptr::NonNull;
use kspin::SpinNoIrq;
use safe_mmio::{
    UniqueMmioPointer, field,
    fields::{ReadOnly, ReadWrite},
};
/// PLIC base address in QEMU virt machine
const PLIC_BASE: PhysAddr = PhysAddr::from_usize(0x0c00_0000);

/// The maximum number of interrupt sources
pub const MAX_DEVICES: usize = 1024;
/// The number of pending bits registers
pub const MAX_PENDING_BITS: usize = 32;

pub const MAX_CONTEXT: usize = 15872;

#[repr(C, align(4096))]
struct ContextLocal {
    priority_threshold: ReadWrite<u32>,
    claim_or_completion: ReadWrite<u32>,
    _reserved: [u8; 4096 - 2 * size_of::<u32>()],
}

/// PLIC registers layout
#[repr(C)]
pub struct PlicRegs {
    /// Interrupt priority registers (1024 entries)
    priority: [ReadWrite<u32>; MAX_DEVICES],
    /// Interrupt pending bits registers
    pending: [ReadOnly<u32>; MAX_PENDING_BITS],
    /// Reserved for future use
    _reserved0: [u8; 0xF80],
    /// Enable bits for sources (context 0)
    enable: [ReadWrite<u32>; MAX_DEVICES * MAX_CONTEXT / MAX_PENDING_BITS],
    /// Reserved for future use
    _reserved1: [u8; 0xE000],
    /// Context local registers
    context_local: [ContextLocal; MAX_CONTEXT],
}

/// PLIC driver
pub struct Plic<'a> {
    regs: SpinNoIrq<UniqueMmioPointer<'a, PlicRegs>>,
}

pub static PLIC: Plic<'static> = Plic {
    regs: SpinNoIrq::new(unsafe {
        UniqueMmioPointer::new(NonNull::new_unchecked(
            phys_to_virt(PLIC_BASE).as_usize() as *mut _
        ))
    }),
};

unsafe impl<'a> Send for Plic<'a> {}
unsafe impl<'a> Sync for Plic<'a> {}

impl Plic<'_> {
    /// Enable an interrupt source
    pub fn enable(&self, hart_id: usize, irq_id: usize) {
        let context_base = 0x80 * (hart_id * 2 + 1);
        info!("context_base: {:#x}", context_base);

        let reg_idx = irq_id / 32;
        let bit_idx = irq_id % 32;

        let pos = context_base / size_of::<u32>() + reg_idx;

        let mut regs = self.regs.lock();
        let val = field!(regs, enable).get(pos).unwrap().read() | (1 << bit_idx);
        field!(regs, enable).get(pos).unwrap().write(val);
    }

    /// Disable an interrupt source
    pub fn disable(&self, irq_id: usize) {
        if irq_id >= MAX_DEVICES || irq_id == 0 {
            return;
        }

        let reg_idx = irq_id / 32;
        let bit_idx = irq_id % 32;

        let mut regs = self.regs.lock();
        let val = field!(regs, enable).get(reg_idx).unwrap().read() & !(1 << bit_idx);
        field!(regs, enable).get(reg_idx).unwrap().write(val);
    }

    /// Set priority for an interrupt source
    pub fn set_priority(&self, irq_id: usize, priority: u8) {
        assert!(priority <= 7);
        if irq_id >= MAX_DEVICES || irq_id == 0 {
            return;
        }

        let mut regs = self.regs.lock();
        field!(regs, priority)
            .get(irq_id)
            .unwrap()
            .write(priority as u32);
        info!("set_priority: {:#x}, {:#x}", irq_id, priority);
    }

    /// Set threshold for a context
    pub fn set_threshold(&self, hart_id: usize, priority: usize, threshold: u32) {
        let pos = hart_id * 2 + priority;
        info!(
            "set_threshold: {:#x}, {:#x}, {:#x}",
            hart_id, priority, threshold
        );
        let mut regs = self.regs.lock();
        let mut context_local = field!(regs, context_local);
        let mut ctx = context_local.get(pos).unwrap();
        info!("set_threshold: {:p}", &ctx);
        field!(ctx, priority_threshold).write(threshold);
        info!("set_threshold: {:#x}", pos);
    }

    /// Claim the highest priority pending interrupt
    pub fn claim(&self, hart_id: usize) -> u32 {
        let pos = hart_id * 2 + 1;
        let mut regs = self.regs.lock();
        let mut context_local = field!(regs, context_local);
        let mut ctx = context_local.get(pos).unwrap();
        field!(ctx, claim_or_completion).read()
    }

    /// Complete an interrupt
    pub fn complete(&self, hart_id: usize, completion: u32) {
        let pos = hart_id * 2 + 1;
        let mut regs = self.regs.lock();
        let mut context_local = field!(regs, context_local);
        let mut ctx = context_local.get(pos).unwrap();
        field!(ctx, claim_or_completion).write(completion);
    }
}
