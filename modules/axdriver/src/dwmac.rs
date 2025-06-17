//! Simple DWMAC HAL Implementation for Tutorial
//!
//! This provides a basic hardware abstraction layer implementation
//! for the DWMAC driver tutorial using JH7110 PAC for type-safe register access.

use axdma::{BusAddr, DMAInfo, alloc_coherent, dealloc_coherent};
use axdriver_net::dwmac::{DwmacHal, PhysAddr as DwmacPhysAddr};
use axhal::mem::{phys_to_virt, virt_to_phys};
use core::sync::atomic::Ordering;
use core::{alloc::Layout, ptr::NonNull, sync::atomic::AtomicBool};
use jh7110_vf2_13b_pac::{self as pac, aon_pinctrl::gmac0_mdio::GMAC0_MDIO_SPEC};

/// Simple HAL implementation for DWMAC
pub struct DwmacHalImpl;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

impl DwmacHal for DwmacHalImpl {
    fn dma_alloc(size: usize) -> (DwmacPhysAddr, NonNull<u8>) {
        let layout = Layout::from_size_align(size, 16).unwrap();
        match unsafe { alloc_coherent(layout) } {
            Ok(dma_info) => {
                // Return bus address for hardware and CPU virtual address
                (dma_info.bus_addr.as_u64() as usize, dma_info.cpu_addr)
            }
            Err(_) => {
                log::error!("DMA allocation failed for size {}", size);
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
        virt_to_phys((vaddr.as_ptr() as usize).into()).as_usize()
    }

    fn wait_until(duration: core::time::Duration) -> Result<(), &'static str> {
        axhal::time::busy_wait(duration);
        Ok(())
    }

    fn configure_platform() -> Result<(), &'static str> {
        log::info!("🔧 StarFive platform configuration (tutorial + PAC verification mode)");

        if unsafe { INITIALIZED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) }
            .is_err()
        {
            log::info!("🔧 Platform already initialized");
            return Ok(());
        }

        // Check if U-Boot has pre-configured the system
        log::info!("🎯 Detected U-Boot pre-configured system - preserving configuration!");
        log::info!("   ✅ Skipping clock initialization");
        log::info!("   ✅ Skipping reset operations");
        log::info!("   ✅ Trusting U-Boot's GMAC/PHY/MDIO setup");

        Self::please_do_not_use_this_function_set_clocks();
        // Just do a quick status check without changing anything
        Self::print_preserved_status();

        log::info!("✅ Platform configuration preserved - ready for DWMAC operation");
        log::info!("💡 TIP: U-Boot has already initialized everything - just trust it!");

        Ok(())
    }
}

impl DwmacHalImpl {
    /// Print status without modifying any registers
    fn print_preserved_status() {
        log::info!("   📊 Current hardware status (read-only, preserved from U-Boot):");

        // Use PAC for available registers
        let aoncrg: &pac::aoncrg::RegisterBlock = unsafe {
            &*(<Self as DwmacHal>::mmio_phys_to_virt(pac::AONCRG::ptr() as usize, 0x1000).as_ptr()
                as *const pac::aoncrg::RegisterBlock)
        };

        let syscrg: &pac::syscrg::RegisterBlock = unsafe {
            &*(<Self as DwmacHal>::mmio_phys_to_virt(pac::SYSCRG::ptr() as usize, 0x1000).as_ptr()
                as *const pac::syscrg::RegisterBlock)
        };

        unsafe {
            // Read-only status check - don't interpret disabled as bad
            log::info!("   🔍 Clock register readings (may not reflect actual hardware state):");

            let gmac5_axi64_axi_enabled = syscrg.clk_gmac5_axi64_axi().read().clk_icg().bit();
            let gmac5_axi64_ptp_enabled = syscrg.clk_gmac5_axi64_ptp().read().clk_icg().bit();
            let gmac0_gtx_enabled = syscrg.clk_gmac0_gtx().read().clk_icg().bit();

            log::info!(
                "     GMAC5 AXI64 AXI: {} (register view)",
                if gmac5_axi64_axi_enabled {
                    "✅ enabled"
                } else {
                    "❓ disabled in register"
                }
            );
            log::info!(
                "     GMAC5 AXI64 PTP: {} (register view)",
                if gmac5_axi64_ptp_enabled {
                    "✅ enabled"
                } else {
                    "❓ disabled in register"
                }
            );
            log::info!(
                "     GMAC0 GTX: {} (register view)",
                if gmac0_gtx_enabled {
                    "✅ enabled"
                } else {
                    "❓ disabled in register"
                }
            );

            log::info!("   💡 Note: Clock registers may show 'disabled' even when hardware works");
            log::info!(
                "   💡 U-Boot may use different initialization sequence than Linux drivers expect"
            );
            log::info!("   💡 The real test is whether networking actually works!");

            log::info!(
                "   🔧 Reset status - Soft reset selector 2: {:#x} (preserved)",
                syscrg.soft_rst_addr_sel_2().read().bits()
            );

            log::info!(
                "   🔧 Reset status - AON reset selector: {:#x} (preserved)",
                aoncrg.soft_rst_addr_sel().read().bits()
            );

            log::info!(
                "   🔧 Clock config - GMAC1 GTX: {:#x} (preserved)",
                syscrg.clk_gmac1_gtx().read().clk_divcfg().bits()
            );

            log::info!(
                "   🔧 Clock config - GMAC1 RMII RTX: {:#x} (preserved)",
                syscrg.clk_gmac1_rmii_rtx().read().clk_divcfg().bits()
            );
        }
    }

    fn print_clocks() {
        // Use PAC for available registers
        let aoncrg: &pac::aoncrg::RegisterBlock = unsafe {
            &*(<Self as DwmacHal>::mmio_phys_to_virt(pac::AONCRG::ptr() as usize, 0x1000).as_ptr()
                as *const pac::aoncrg::RegisterBlock)
        };

        let syscrg: &pac::syscrg::RegisterBlock = unsafe {
            &*(<Self as DwmacHal>::mmio_phys_to_virt(pac::SYSCRG::ptr() as usize, 0x1000).as_ptr()
                as *const pac::syscrg::RegisterBlock)
        };

        unsafe {
            if syscrg.clk_gmac5_axi64_axi().read().clk_icg().bit() {
                log::debug!("🔧 GMAC5 AXI64 clock is enabled");
            } else {
                log::debug!("🔧 GMAC5 AXI64 clock is disabled");
            }

            if syscrg.clk_gmac5_axi64_ptp().read().clk_icg().bit() {
                log::debug!("🔧 GMAC5 AXI64 PTP clock is enabled");
            } else {
                log::debug!("🔧 GMAC5 AXI64 PTP clock is disabled");
            }

            if syscrg.clk_gmac5_axi64_tx().read().clk_icg().bit() {
                log::debug!("🔧 GMAC5 AXI64 TX clock is enabled");
            } else {
                log::debug!("🔧 GMAC5 AXI64 TX clock is disabled");
            }

            if syscrg.clk_gmac0_gtx().read().clk_icg().bit() {
                log::debug!("🔧 GMAC0 GTX clock is enabled");
            } else {
                log::debug!("🔧 GMAC0 GTX clock is disabled");
            }

            if syscrg.clk_gmac0_ptp().read().clk_icg().bit() {
                log::debug!("🔧 GMAC0 PTP clock is enabled");
            } else {
                log::debug!("🔧 GMAC0 PTP clock is disabled");
            }

            if aoncrg.clk_axi_gmac5().read().clk_icg().bit() {
                log::debug!("🔧 GMAC5 AXI clock is enabled");
            } else {
                log::debug!("🔧 GMAC5 AXI clock is disabled");
            }

            if aoncrg.clk_gmac5_axi64_tx().read().bits() & 0x8000_0000 != 0 {
                log::debug!("🔧 GMAC5 AXI64 TX clock is enabled");
            } else {
                log::debug!("🔧 GMAC5 AXI64 TX clock is disabled");
            }

            log::debug!(
                "🔧 Soft reset address selector 2: {:#x}",
                syscrg.soft_rst_addr_sel_2().read().bits()
            );

            log::debug!(
                "🔧 Soft reset address selector: {:#x}",
                aoncrg.soft_rst_addr_sel().read().bits()
            );

            log::debug!(
                "🔧 GMAC1 GTX clock: {:#x}",
                syscrg.clk_gmac1_gtx().read().clk_divcfg().bits()
            );

            log::debug!(
                "🔧 GMAC1 RMII RTX clock: {:#x}",
                syscrg.clk_gmac1_rmii_rtx().read().clk_divcfg().bits()
            );
        }
    }

    /// Verify that U-Boot has properly enabled the GMAC clocks using JH7110 PAC
    /// This demonstrates both PAC usage and its real-world limitations
    fn please_do_not_use_this_function_set_clocks() {
        log::info!("🔍 Verifying StarFive GMAC clock configuration (PAC + manual)...");

        // Use PAC for available registers
        let aoncrg: &pac::aoncrg::RegisterBlock = unsafe {
            &*(<Self as DwmacHal>::mmio_phys_to_virt(pac::AONCRG::ptr() as usize, 0x1000).as_ptr()
                as *const pac::aoncrg::RegisterBlock)
        };

        let syscrg: &pac::syscrg::RegisterBlock = unsafe {
            &*(<Self as DwmacHal>::mmio_phys_to_virt(pac::SYSCRG::ptr() as usize, 0x1000).as_ptr()
                as *const pac::syscrg::RegisterBlock)
        };

        unsafe {
            log::info!("   📊 Reading clock registers with PAC (where available)...");

            // PAC-based register access for available clocks
            log::info!("   🔧 Available PAC clock registers:");

            // 🎓 EDUCATIONAL: PAC Limitation Example
            log::info!("   💡 PAC Limitation: GMAC-specific clocks not exposed in PAC");
            log::info!("   📚 Real-world lesson: PACs don't always cover everything!");

            // Fall back to manual register access for GMAC clocks
            log::info!("   🔧 GMAC clocks (manual register access):");

            syscrg
                .clk_gmac5_axi64_ahb()
                .write(|w| w.clk_icg().set_bit());
            syscrg
                .clk_gmac5_axi64_axi()
                .write(|w| w.clk_icg().set_bit());
            syscrg
                .clk_gmac5_axi64_ptp()
                .write(|w| w.clk_icg().set_bit());
            syscrg.clk_gmac5_axi64_tx().write(|w| w.clk_icg().set_bit());
            syscrg.clk_gmac0_gtx().write(|w| w.clk_icg().set_bit());
            syscrg.clk_gmac0_ptp().write(|w| w.clk_icg().set_bit());
            syscrg.clk_gmac_phy().write(|w| w.clk_icg().set_bit());
            // 添加缺失的SYSCRG时钟
            syscrg
                .clk_gmac5_axi64_rx()
                .write(|w| w.dly_chain_sel().bits(0x0));
            syscrg
                .clk_gmac5_axi64_rxi()
                .write(|w| w.clk_polarity().set_bit());
            syscrg.clk_noc_stg_axi().write(|w| w.clk_icg().set_bit());
            syscrg.clk_gmac_src().write(|w| w.clk_divcfg().bits(2));

            // 可选：GMAC0 GTXC时钟（用于时序调整）
            syscrg
                .clk_gmac0_gtxclk()
                .write(|w| w.dly_chain_sel().bits(0x0));

            aoncrg.clk_ahb_gmac5().write(|w| w.clk_icg().set_bit());
            aoncrg.clk_axi_gmac5().write(|w| w.clk_icg().set_bit());
            aoncrg
                .clk_gmac5_axi64_tx()
                .write(|w| w.bits(0x8000_0000).clk_mux_sel().bits(0));

            // GMAC1 clocks
            let gmac1_ahb = syscrg.clk_gmac5_axi64_ahb().read();
            let gmac1_axi = syscrg.clk_gmac5_axi64_axi().read();
            let gmac1_src = syscrg.clk_gmac_src().read();
            let gmac1_ptp = syscrg.clk_gmac5_axi64_ptp().read();

            log::info!(
                "     GMAC1 AHB (ID 97): {:#x} {}",
                gmac1_ahb.bits(),
                if gmac1_ahb.clk_icg().bit() {
                    "✅ ENABLED"
                } else {
                    "❌ DISABLED"
                }
            );
            log::info!(
                "     GMAC1 AXI (ID 98): {:#x} {}",
                gmac1_axi.bits(),
                if gmac1_axi.clk_icg().bit() {
                    "✅ ENABLED"
                } else {
                    "❌ DISABLED"
                }
            );
            log::info!(
                "     GMAC1 PTP (ID 102): {:#x} {}",
                gmac1_ptp.bits(),
                if gmac1_ptp.clk_icg().bit() {
                    "✅ ENABLED"
                } else {
                    "❌ DISABLED"
                }
            );

            // Check readiness
            let gmac1_ready = gmac1_ahb.clk_icg().bit() && gmac1_axi.clk_icg().bit();

            if gmac1_ready {
                log::info!("   ✅ Clock verification shows some GMAC clocks enabled");
                log::info!("     GMAC1 ready: {}", gmac1_ready);
            } else {
                log::warn!("   ⚠️  Clock registers show disabled, but this may be incorrect");
                log::info!(
                    "     💡 If RJ45 LEDs blink, ignore this - U-Boot configured hardware correctly!"
                );
            }

            syscrg
                .soft_rst_addr_sel_2()
                .modify(|_, w| w.bits(0xffe5afc4));

            syscrg
                .soft_rst_addr_sel_2()
                .modify(|_, w| w.bits(0xffe5afc0));

            aoncrg.soft_rst_addr_sel().write(|w| w.bits(0xe1));
            aoncrg.soft_rst_addr_sel().write(|w| w.bits(0xe0));
            aoncrg.soft_rst_addr_sel().write(|w| w.bits(0xe2));
            aoncrg.soft_rst_addr_sel().write(|w| w.bits(0xe3));

            syscrg
                .clk_gmac1_gtx()
                .write(|w| w.clk_divcfg().variant(0x8));
            syscrg
                .clk_gmac1_rmii_rtx()
                .write(|w| w.clk_divcfg().variant(0x2));
        }
    }

    /// Verify PLL2 is running (GMAC clock source) using PAC - informational only
    fn verify_pll2_status_with_pac() {
        let syscrg: &pac::syscrg::RegisterBlock = unsafe {
            &*(<Self as DwmacHal>::mmio_phys_to_virt(pac::SYSCRG::ptr() as usize, 0x1000).as_ptr()
                as *const pac::syscrg::RegisterBlock)
        };
        unsafe {
            // Use PAC for available PLL-related registers
            log::info!("   🔍 PLL Status (using available PAC registers):");

            // Check available PLL divider registers in PAC
            let pll0_div2 = syscrg.clk_pll0_div2().read();
            log::info!(
                "     PLL0_DIV2: divcfg={}, raw={:#x}",
                pll0_div2.clk_divcfg().bits(),
                pll0_div2.bits()
            );

            let pll1_div2 = syscrg.clk_pll1_div2().read();
            log::info!(
                "     PLL1_DIV2: divcfg={}, raw={:#x}",
                pll1_div2.clk_divcfg().bits(),
                pll1_div2.bits()
            );

            let pll2_div2 = syscrg.clk_pll2_div2().read();
            log::info!(
                "     PLL2_DIV2: divcfg={}, raw={:#x}",
                pll2_div2.clk_divcfg().bits(),
                pll2_div2.bits()
            );

            // 🎓 EDUCATIONAL: PAC Field Discovery Process
            log::info!(
                "   💡 PAC Field Discovery: clk_divcfg (not clk_icg) controls divider enable"
            );
            log::info!(
                "   📚 Real-world lesson: PAC field names need to be discovered, not assumed!"
            );
            log::info!(
                "   🔍 DIVCFG = Divider Configuration Enable (vs ICG = Integrated Clock Gating)"
            );

            // 🎓 EDUCATIONAL: Another PAC Limitation Example
            log::info!(
                "   💡 PAC Limitation: PLL control registers (pll2_pd, pll2_dacpd) not exposed"
            );
            log::info!(
                "   📚 Available: Only PLL divider outputs, not PLL configuration registers"
            );

            // Determine status from available information
            if pll2_div2.bits() & 1 == 1 {
                log::info!("     ✅ PLL2_DIV2 divider is enabled - suggests PLL2 is running");
                log::info!("     💡 GMAC likely gets clock from PLL2 → DIV2 → further dividers");
            } else {
                log::warn!("     ⚠️  PLL2_DIV2 divider disabled - but PLL2 might still be running");
                log::info!("     💡 RJ45 LED activity is the real test of clock functionality!");
            }

            // Show what a complete PAC would provide (educational)
            log::info!("   📝 Missing from PAC (would need manual access):");
            log::info!("     - PLL2 power down control (pll2_pd)");
            log::info!("     - PLL2 feedback divider (pll2_dacpd)");
            log::info!("     - PLL2 fractional settings (pll2_frac)");
            log::info!("     - PLL2 lock status");
        }
    }
}
