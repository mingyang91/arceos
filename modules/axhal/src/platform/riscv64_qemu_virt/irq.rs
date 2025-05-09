//! IRQ handling using PLIC for QEMU virt machine

use super::plic::{MAX_DEVICES, PLIC, Plic};
use crate::irq::IrqHandler;
use lazyinit::LazyInit;
use log::{info, trace};
use riscv::register::sie;

/// `Interrupt` bit in `scause`
pub(super) const INTC_IRQ_BASE: usize = 1 << (usize::BITS - 1);

/// Supervisor software interrupt in `scause`
#[allow(unused)]
pub(super) const S_SOFT: usize = INTC_IRQ_BASE + 1;

/// Supervisor timer interrupt in `scause`
pub(super) const S_TIMER: usize = INTC_IRQ_BASE + 5;

/// Supervisor external interrupt in `scause`
pub(super) const S_EXT: usize = INTC_IRQ_BASE + 9;

static TIMER_HANDLER: LazyInit<IrqHandler> = LazyInit::new();

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = MAX_DEVICES;

/// The timer IRQ number (supervisor timer interrupt in `scause`).
pub const TIMER_IRQ_NUM: usize = S_TIMER;

/// IRQ numbers for virtio devices
pub const VIRTIO_NET_IRQ: usize = 2;
pub const VIRTIO_BLK_IRQ: usize = 3;

macro_rules! with_cause {
    ($cause: expr, @TIMER => $timer_op: expr, @EXT => $ext_op: expr $(,)?) => {
        match $cause {
            S_TIMER => $timer_op,
            _ => $ext_op,
        }
    };
}

/// Enables or disables the given IRQ.
pub fn set_enable(irq_num: usize, enabled: bool) {
    let hart_id = crate::cpu::this_cpu_id();
    // For other IRQs, enable/disable in PLIC
    PLIC.enable(hart_id, irq_num);
}

/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
pub fn register_handler(scause: usize, handler: IrqHandler) -> bool {
    info!("cause: {:#x}", scause);
    info!("cause & !INTC_IRQ_BASE: {:#x}", scause & !INTC_IRQ_BASE);
    with_cause!(
        scause,
        @TIMER => if !TIMER_HANDLER.is_inited() {
            TIMER_HANDLER.init_once(handler);
            true
        } else {
            false
        },
        @EXT => crate::irq::register_handler_common(scause & !INTC_IRQ_BASE, handler),
    )
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(scause: usize) {
    with_cause!(
        scause,
        @TIMER => {
            trace!("IRQ: timer");
            TIMER_HANDLER();
        },
        @EXT => {
            // Handle external interrupt from PLIC
            let hart_id = crate::cpu::this_cpu_id();
            let irq_num = PLIC.claim(hart_id);
            trace!("IRQ: external {}", irq_num);
            crate::irq::dispatch_irq_common(irq_num as usize);
            PLIC.complete(hart_id, irq_num);
        }
    );
}

pub(super) fn init_percpu() {
    // Enable all types of interrupts
    unsafe {
        sie::set_ssoft();
        sie::set_stimer();
        sie::set_sext();
    }
}
