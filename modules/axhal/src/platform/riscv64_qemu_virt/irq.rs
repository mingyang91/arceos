//! IRQ handling using PLIC for QEMU virt machine

use crate::irq::IrqHandler;
use crate::mem::{PhysAddr, phys_to_virt};
use core::num::NonZeroU32;
use lazyinit::LazyInit;
use log::{info, trace};
use riscv::register::sie;
use riscv_plic::{HartContext, InterruptSource, Plic};

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
pub const MAX_IRQ_COUNT: usize = 1024;

/// The timer IRQ number (supervisor timer interrupt in `scause`).
pub const TIMER_IRQ_NUM: usize = S_TIMER;

pub static PLIC: LazyInit<Plic> = LazyInit::new();

#[derive(Debug, Clone, Copy)]
struct HartCtx(usize);

impl HartCtx {
    fn this_hart_supervisor() -> Self {
        Self(crate::cpu::this_cpu_id() * 2 + 1)
    }

    fn this_hart_machine() -> Self {
        Self(crate::cpu::this_cpu_id() * 2)
    }
}

impl HartContext for HartCtx {
    fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
struct Irq(u32);

impl InterruptSource for Irq {
    fn id(self) -> NonZeroU32 {
        NonZeroU32::new(self.0).unwrap()
    }
}

/// Enables or disables the given IRQ.
pub fn set_enable(irq_num: usize, enabled: bool) {
    let hart_ctx = HartCtx::this_hart_supervisor();
    let irq = Irq(irq_num as u32);
    if enabled {
        // For other IRQs, enable/disable in PLIC
        PLIC.enable(irq, hart_ctx);
        PLIC.set_priority(irq, 1);
    } else {
        PLIC.disable(irq, hart_ctx);
    }
}

/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
pub fn register_handler(irq_num: usize, handler: IrqHandler) -> bool {
    info!("register_handler: {}", irq_num);

    match irq_num {
        TIMER_IRQ_NUM => {
            if !TIMER_HANDLER.is_inited() {
                TIMER_HANDLER.init_once(handler);
                true
            } else {
                false
            }
        }
        _ => crate::irq::register_handler_common(irq_num, handler),
    }
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(scause: usize) {
    match scause {
        TIMER_IRQ_NUM => {
            trace!("IRQ: timer");
            TIMER_HANDLER();
        }
        S_EXT => {
            // Handle external interrupt from PLIC
            let hart_ctx = HartCtx::this_hart_supervisor();
            let Some(irq) = PLIC.claim(hart_ctx) else {
                error!("IRQ: external claim failed");
                return;
            };
            crate::irq::dispatch_irq_common(irq.get() as usize);
            PLIC.complete(hart_ctx, Irq(irq.get()));
        }
        _ => {
            panic!("IRQ: unknown {}", scause);
        }
    }
}

const PLIC_BASE: usize = 0x0c00_0000;

fn init_plic() {
    let base = phys_to_virt(PhysAddr::from_usize(PLIC_BASE));
    let regs = base.as_mut_ptr();
    PLIC.init_once(Plic::new(regs));
}

pub(super) fn init_percpu() {
    init_plic();
    let hart_ctx_machine = HartCtx::this_hart_machine();
    PLIC.set_threshold(hart_ctx_machine, 1);
    let hart_ctx_supervisor = HartCtx::this_hart_supervisor();
    PLIC.set_threshold(hart_ctx_supervisor, 0);

    // Enable all types of interrupts
    unsafe {
        sie::set_ssoft();
        sie::set_stimer();
        sie::set_sext();
    }
}
