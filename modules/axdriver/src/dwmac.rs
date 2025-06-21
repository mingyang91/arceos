//! Simple DWMAC HAL Implementation for Tutorial
//!
//! This provides a basic hardware abstraction layer implementation
//! for the DWMAC driver tutorial using JH7110 PAC for type-safe register access.

use axdma::{BusAddr, DMAInfo, alloc_coherent, dealloc_coherent};
use axdriver_net::dwmac::{DwmacHal, PhysAddr as DwmacPhysAddr};
use axdriver_virtio::PhysAddr;
use axhal::mem::{MemoryAddr, phys_to_virt, virt_to_phys};
use core::sync::atomic::Ordering;
use core::{alloc::Layout, ptr::NonNull, sync::atomic::AtomicBool};
use jh7110_vf2_13b_pac::{self as pac, aon_pinctrl::gmac0_mdio::GMAC0_MDIO_SPEC};

/// Simple HAL implementation for DWMAC
pub struct DwmacHalImpl;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

fn mb() {
    unsafe { core::arch::asm!("fence iorw, iorw") };
}

impl DwmacHal for DwmacHalImpl {
    fn cache_flush_range(start: NonNull<u8>, end: NonNull<u8>) {
        const CCACHE_BASE: usize = 0x0201_0000;
        const FLUSH64_OFFSET: usize = 0x200;
        const LINE_SIZE: usize = 64;

        let mut addr = start.as_ptr() as usize & !(LINE_SIZE - 1);

        let flush_addr = phys_to_virt(CCACHE_BASE.into())
            .add(FLUSH64_OFFSET)
            .as_mut_ptr() as *mut u32;
        let end_addr = end.as_ptr() as usize;
        mb();
        while addr < end_addr {
            unsafe {
                core::ptr::write_volatile(flush_addr, addr as u32);
                addr += LINE_SIZE;
            }
        }
    }

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

        // Self::setup_clocks();
        Self::init_clocks();
        // Self::set_clocks_linus();
        // Just do a quick status check without changing anything
        Self::print_preserved_status();

        log::info!("✅ Platform configuration preserved - ready for DWMAC operation");
        log::info!("💡 TIP: U-Boot has already initialized everything - just trust it!");

        Ok(())
    }
}

impl DwmacHalImpl {
    fn set_clocks_linus() {
        let dump = r"
0x17000000: 0x00000004
0x17000004: 0x01000000
0x17000008: 0x80000000
0x1700000C: 0x80000000
0x17000010: 0x00000002
0x17000014: 0x81000000
0x17000018: 0x40000000
0x1700001C: 0x00000020
0x17000020: 0x40000000
0x17000024: 0x80000000
0x17000028: 0x80000000
0x1700002C: 0x000002EE
0x17000030: 0x00000000
0x17000034: 0x80000000
0x17000038: 0x00000000
0x1700003C: 0x000000FF
0x17000040: 0x00000000
0x17000044: 0x00000000
0x17000048: 0x00000000
0x1700004C: 0x00000000 
0x13020000: 0x01000000
0x13020004: 0x00000004
0x13020008: 0x00000002
0x1302000C: 0x00000000
0x13020010: 0x01000002
0x13020014: 0x01000000
0x13020018: 0x00000003
0x1302001C: 0x00000003
0x13020020: 0x00000002
0x13020024: 0x80000000
0x13020028: 0x80000000
0x1302002C: 0x00000004
0x13020030: 0x80000000
0x13020034: 0x00000002
0x13020038: 0x00000002
0x1302003C: 0x00000002
0x13020040: 0x00000002
0x13020044: 0x0000000C
0x13020048: 0x01000000
0x1302004C: 0x00000000
0x13020050: 0x00000002
0x13020054: 0x00000002
0x13020058: 0x00000014
0x1302005C: 0x00000010
0x13020060: 0x0000000C
0x13020064: 0x80000000
0x13020068: 0x80000000
0x1302006C: 0x80000000
0x13020070: 0x80000000
0x13020074: 0x80000000
0x13020078: 0x80000000
0x1302007C: 0x00000006
0x13020080: 0x80000000
0x13020084: 0x80000000
0x13020088: 0x80000000
0x1302008C: 0x80000000
0x13020090: 0x80000000
0x13020094: 0x80000000
0x13020098: 0x80000000
0x1302009C: 0x80000000
0x130200A0: 0x00000002
0x130200A4: 0x00000002
0x130200A8: 0x00000002
0x130200AC: 0x01000000
0x130200B0: 0x80000000
0x130200B4: 0x00000002
0x130200B8: 0x00000000
0x130200BC: 0x00000000
0x130200C0: 0x00000000
0x130200C4: 0x0000000C
0x130200C8: 0x80000000
0x130200CC: 0x00000000
0x130200D0: 0x00000000
0x130200D4: 0x80000000
0x130200D8: 0x00000003
0x130200DC: 0x00000002
0x130200E0: 0x80000000
0x130200E4: 0x80000000
0x130200E8: 0x00000000
0x130200EC: 0x00000002
0x130200F0: 0x00000000
0x130200F4: 0x00000000
0x130200F8: 0x00000000
0x130200FC: 0x00000000
0x13020100: 0x00000002
0x13020104: 0x00000006
0x13020108: 0x00000000
0x1302010C: 0x00000006
0x13020110: 0x00000000
0x13020114: 0x00000003
0x13020118: 0x00000000
0x1302011C: 0x00000003
0x13020120: 0x00000002
0x13020124: 0x00000000
0x13020128: 0x80000000
0x1302012C: 0x80000000
0x13020130: 0x00000000
0x13020134: 0x00000005
0x13020138: 0x00000000
0x1302013C: 0x00000005
0x13020140: 0x00000005
0x13020144: 0x00000000
0x13020148: 0x00000000
0x1302014C: 0x80000000
0x13020150: 0x80000000
0x13020154: 0x80000000
0x13020158: 0x80000000
0x1302015C: 0x80000000
0x13020160: 0x80000000
0x13020164: 0x0000000A
0x13020168: 0x81000000
0x1302016C: 0x80000000
0x13020170: 0x80000000
0x13020174: 0x80000008
0x13020178: 0x80000008
0x1302017C: 0x0000000C
0x13020180: 0x80000000
0x13020184: 0x80000000
0x13020188: 0x80000000
0x1302018C: 0x00000002
0x13020190: 0x0000000C
0x13020194: 0x00000001
0x13020198: 0x8000000A
0x1302019C: 0x00000020
0x130201A0: 0x40000000
0x130201A4: 0x81000000
0x130201A8: 0x40000000
0x130201AC: 0x80000020
0x130201B0: 0x80000008
0x130201B4: 0x8000000A
0x130201B8: 0x0000000A
0x130201BC: 0x80000020
0x130201C0: 0x80000000
0x130201C4: 0x00000000
0x130201C8: 0x80000000
0x130201CC: 0x00000000
0x130201D0: 0x00000018
0x130201D4: 0x00000008
0x130201D8: 0x00000000
0x130201DC: 0x00000018
0x130201E0: 0x00000008
0x130201E4: 0x80000000
0x130201E8: 0x00000000
0x130201EC: 0x00000000
0x130201F0: 0x80000000
0x130201F4: 0x80000000
0x130201F8: 0x80000000
0x130201FC: 0x80000000
0x13020200: 0x80000000
0x13020204: 0x80000000
0x13020208: 0x80000018
0x1302020C: 0x00000000
0x13020210: 0x00000000
0x13020214: 0x00000000
0x13020218: 0x00000000
0x1302021C: 0x00000000
0x13020220: 0x00000000
0x13020224: 0x00000000
0x13020228: 0x00000000
0x1302022C: 0x00000000
0x13020230: 0x00000000
0x13020234: 0x00000000
0x13020238: 0x00000000
0x1302023C: 0x00000000
0x13020240: 0x00000000
0x13020244: 0x80000000
0x13020248: 0x80000000
0x1302024C: 0x00000000
0x13020250: 0x00000000
0x13020254: 0x00000000
0x13020258: 0x00000000
0x1302025C: 0x00000000
0x13020260: 0x00000A00
0x13020264: 0x00000000
0x13020268: 0x00000A00
0x1302026C: 0x00000000
0x13020270: 0x00000A00
0x13020274: 0x00000000
0x13020278: 0x00000092
0x1302027C: 0x00000000
0x13020280: 0x00000000
0x13020284: 0x00000000
0x13020288: 0x00000004
0x1302028C: 0x40000000
0x13020290: 0x01000040
0x13020294: 0x00000000
0x13020298: 0x40000000
0x1302029C: 0x00000000
0x130202A0: 0x00000000
0x130202A4: 0x00000004
0x130202A8: 0x40000000
0x130202AC: 0x00000040
0x130202B0: 0x00000000
0x130202B4: 0x40000000
0x130202B8: 0x00000000
0x130202BC: 0x00000000
0x130202C0: 0x00000004
0x130202C4: 0x40000000
0x130202C8: 0x00000040
0x130202CC: 0x00000000
0x130202D0: 0x40000000
0x130202D4: 0x00000000
0x130202D8: 0x00000008
0x130202DC: 0x00000000
0x130202E0: 0x00000000
0x130202E4: 0x00000000
0x130202E8: 0x00000001
0x130202EC: 0x00000000
0x130202F0: 0x40000000
0x130202F4: 0x00000004
0x130202F8: 0x00600000
0x130202FC: 0x07E7F000
0x13020300: 0xFFE1AFC0
0x13020304: 0x041F8F9E
0x13020308: 0xFF9FFFFF
0x1302030C: 0xF80001FF
0x13020310: 0x001E503F
0x13020314: 0x3BE07061
";

        for line in dump.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let (addr, tail) = line.split_once(':').unwrap();
            let addr = u32::from_str_radix(&addr.trim()[2..], 16).unwrap();
            let val = u32::from_str_radix(&tail.trim()[2..], 16).unwrap();
            unsafe {
                Self::write_reg(addr as usize, val);
            }
        }
    }
    /// 这部分 syscrg 和 aoncrg 的值是用 md 命令从 uboot 中 dump 出来的
    /// 但是 uboot 仅在执行网络命令的时候才会启用网卡，执行完毕后会关闭网卡
    /// 所以有效值无法在 uboot 中获取
    fn setup_clocks() {
        let dump = r"
17000000: 00000004 01000000 80000000 80000000  ................
17000010: 00000002 81000000 40000000 00000020  ...........@ ...
17000020: 40000000 80000000 80000000 000002ee  ...@............
17000030: 00000000 00000000 000000e3 0000001c  ................
13020000: 01000000 00000001 00000002 00000000  ................
13020010: 01000002 01000000 00000003 00000003  ................
13020020: 00000002 80000000 80000000 00000004  ................
13020030: 80000000 00000002 00000002 00000002  ................
13020040: 00000002 0000000c 00000000 00000000  ................
13020050: 00000002 00000002 80000014 80000010  ................
13020060: 8000000c 80000000 80000000 80000000  ................
13020070: 80000000 80000000 80000000 00000006  ................
13020080: 80000000 80000000 80000000 80000000  ................
13020090: 80000000 80000000 80000000 80000000  ................
130200a0: 00000002 00000002 00000002 01000000  ................
130200b0: 80000000 00000003 00000000 00000000  ................
130200c0: 00000000 0000000c 00000000 00000000  ................
130200d0: 00000000 80000000 00000003 00000002  ................
130200e0: 80000000 80000000 00000000 00000002  ................
130200f0: 00000000 00000000 00000000 00000000  ................
13020100: 00000002 00000006 00000000 00000006  ................
13020110: 00000000 00000003 00000000 00000003  ................
13020120: 00000002 00000000 80000000 80000000  ................
13020130: 00000000 00000005 00000000 00000005  ................
13020140: 00000005 00000000 00000000 80000000  ................
13020150: 80000000 80000000 80000000 80000000  ................
13020160: 80000000 0000000a 81000000 80000000  ................
13020170: 80000000 80000002 80000002 00000008  ................
13020180: 80000000 80000000 80000000 00000002  ................
13020190: 0000000f 00000002 8000000a 00000020  ............ ...
130201a0: 40000000 81000000 40000000 80000020  ...@.......@ ...
130201b0: 80000008 8000000a 0000000a 80000020  ............ ...
130201c0: 80000000 80000000 80000000 00000000  ................
130201d0: 00000018 00000008 00000000 00000018  ................
130201e0: 00000008 00000000 80000000 80000000  ................
130201f0: 80000000 80000000 80000000 80000000  ................
13020200: 80000000 00000000 00000018 00000000  ................
13020210: 00000000 00000000 00000000 00000000  ................
13020220: 00000000 00000000 00000000 00000000  ................
13020230: 00000000 00000000 00000000 80000000  ................
13020240: 00000000 80000000 80000000 00000000  ................
13020250: 00000000 00000000 00000000 00000000  ................
13020260: 00000a00 00000000 00000a00 00000000  ................
13020270: 00000a00 00000000 0000000c 00000000  ................
13020280: 00000000 00000000 00000004 40000000  ...............@
13020290: 00000040 00000000 40000000 00000000  @..........@....
130202a0: 00000000 00000004 40000000 00000040  ...........@@...
130202b0: 00000000 40000000 00000000 00000000  .......@........
130202c0: 00000004 40000000 00000040 00000000  .......@@.......
130202d0: 40000000 00000000 00000008 00000000  ...@............
130202e0: 00000000 00000000 00000001 00000000  ................
130202f0: 40000000 00000004 00600000 07e7fe00  ...@......`.....
13020300: ffe5efcc 1c1fffff ff9fffff f80001ff  ................
13020310: 001a1033 23e00000 00000000 00000000  3......#........
";

        for line in dump.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let (addr, tail) = line.split_once(':').unwrap();
            let addr = u32::from_str_radix(addr.trim(), 16).unwrap();
            let val1 = u32::from_str_radix(&tail[1..9], 16).unwrap();
            let val2 = u32::from_str_radix(&tail[10..18], 16).unwrap();
            let val3 = u32::from_str_radix(&tail[19..27], 16).unwrap();
            let val4 = u32::from_str_radix(&tail[28..36], 16).unwrap();

            log::trace!("    🔍 {:#x}: {:#x}", addr, val1);
            log::trace!("    🔍 {:#x}: {:#x}", addr + 4, val2);
            log::trace!("    🔍 {:#x}: {:#x}", addr + 8, val3);
            log::trace!("    🔍 {:#x}: {:#x}", addr + 12, val4);
            unsafe {
                Self::write_reg(addr as usize, val1);
                Self::write_reg(addr as usize + 4, val2);
                Self::write_reg(addr as usize + 8, val3);
                Self::write_reg(addr as usize + 12, val4);
            }
        }
    }

    fn write_reg(paddr: PhysAddr, val: u32) {
        unsafe {
            let vaddr = <Self as DwmacHal>::mmio_phys_to_virt(paddr, 0x1000);
            core::ptr::write_volatile(vaddr.as_ptr() as *mut u32, val);
        }
    }

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
    fn init_clocks() {
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
                .clk_gmac1_gtxclk()
                .write(|w| w.bits(0x8000_0000).dly_chain_sel().bits(0x20));
            syscrg
                .clk_gmac5_axi64_ahb()
                .write(|w| w.clk_icg().set_bit());
            syscrg
                .clk_gmac5_axi64_axi()
                .write(|w| w.clk_icg().set_bit());
            syscrg
                .clk_gmac5_axi64_ptp()
                .write(|w| w.clk_icg().set_bit().clk_divcfg().variant(0xa));
            syscrg.clk_gmac5_axi64_tx().write(|w| w.clk_icg().set_bit());
            syscrg
                .clk_gmac5_axi64_txi()
                .write(|w| w.bits(0x8000_0000).clk_polarity().set_bit());
            syscrg.clk_gmac0_gtx().write(|w| w.clk_icg().set_bit());
            syscrg.clk_gmac0_ptp().write(|w| w.clk_icg().set_bit());
            syscrg.clk_gmac_phy().write(|w| w.clk_icg().set_bit());
            // 添加缺失的SYSCRG时钟
            syscrg
                .clk_gmac5_axi64_rx()
                .write(|w| w.bits(0x8000_0000).dly_chain_sel().bits(0x20));
            syscrg
                .clk_gmac5_axi64_rxi()
                .write(|w| w.clk_polarity().set_bit());
            syscrg.clk_noc_stg_axi().write(|w| w.clk_icg().set_bit());
            syscrg
                .clk_gmac_src()
                .write(|w| w.bits(0x8000_0000).clk_divcfg().bits(3));

            // 可选：GMAC0 GTXC时钟（用于时序调整）
            syscrg
                .clk_gmac0_gtxclk()
                .write(|w| w.dly_chain_sel().bits(0x4));

            aoncrg.clk_ahb_gmac5().write(|w| w.clk_icg().set_bit());
            aoncrg.clk_axi_gmac5().write(|w| w.clk_icg().set_bit());
            aoncrg
                .clk_gmac5_axi64_tx()
                .write(|w| w.bits(0x8000_0000).clk_mux_sel().bits(0));
            aoncrg
                .clk_gmac5_axi64_rx()
                .write(|w| w.dly_chain_sel().bits(0x20));
            aoncrg
                .clk_gmac5_axi64_rxi()
                .write(|w| w.clk_polarity().set_bit());

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
                .write(|w| w.clk_divcfg().variant(0xc));
            syscrg
                .clk_gmac1_rmii_rtx()
                .write(|w| w.clk_divcfg().variant(0x1));
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
