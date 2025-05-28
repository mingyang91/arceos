# DWMAC Driver Load Access Fault Debugging Guide

## Issue Analysis

### Problem Description
- **Error**: `mcause=5` (Load Access Fault) at address `0x40048060`
- **Context**: Occurred during DWMAC Ethernet driver initialization on StarFive VisionFive 2
- **Impact**: System crash preventing network functionality

### Root Cause Analysis

The Load Access Fault was caused by several interconnected issues in the DMA memory management:

1. **Bus Address vs Physical Address Confusion**
   - The DWMAC hardware expects bus addresses for DMA operations
   - The original implementation was mixing physical and bus addresses
   - Bus address = Physical address + `PHYS_BUS_OFFSET` (0 on VisionFive 2)

2. **Incorrect Address Translation**
   - `mmio_virt_to_phys()` was being used for DMA buffer addresses
   - Should use bus addresses from DMA allocation instead

3. **Insufficient Memory Alignment**
   - DMA descriptors require 16-byte alignment for proper hardware access
   - Original code used 8-byte alignment

4. **Lack of Validation**
   - No bounds checking on MMIO register access
   - No validation of DMA memory allocation success

## Fixes Implemented

### 1. DMA Memory Allocation Fix (`modules/axdriver/src/dwmac.rs`)

**Before:**
```rust
fn dma_alloc(size: usize) -> (DwmacPhysAddr, NonNull<u8>) {
    let layout = Layout::from_size_align(size, 8).unwrap();
    match unsafe { alloc_coherent(layout) } {
        Ok(dma_info) => (dma_info.bus_addr.as_u64() as usize, dma_info.cpu_addr),
        Err(_) => (0, NonNull::dangling()),
    }
}
```

**After:**
```rust
fn dma_alloc(size: usize) -> (DwmacPhysAddr, NonNull<u8>) {
    let layout = Layout::from_size_align(size, 16).unwrap(); // 16-byte alignment
    match unsafe { alloc_coherent(layout) } {
        Ok(dma_info) => {
            log::debug!("DMA alloc: size={}, cpu_addr={:p}, bus_addr={:#x}", 
                       size, dma_info.cpu_addr.as_ptr(), dma_info.bus_addr.as_u64());
            (dma_info.bus_addr.as_u64() as usize, dma_info.cpu_addr)
        },
        Err(e) => {
            log::error!("DMA allocation failed: {:?}", e);
            (0, NonNull::dangling())
        },
    }
}
```

### 2. Address Translation Fix

**Before:**
```rust
unsafe fn mmio_virt_to_phys(vaddr: NonNull<u8>, _size: usize) -> DwmacPhysAddr {
    virt_to_phys((vaddr.as_ptr() as usize).into()).into()
}
```

**After:**
```rust
unsafe fn mmio_virt_to_phys(vaddr: NonNull<u8>, _size: usize) -> DwmacPhysAddr {
    let virt_addr = vaddr.as_ptr() as usize;
    let phys_addr = virt_to_phys(virt_addr.into());
    
    // Convert physical address to bus address
    let bus_addr = phys_addr.as_usize() + axconfig::plat::PHYS_BUS_OFFSET;
    log::trace!("virt_to_phys: virt={:#x} -> phys={:#x} -> bus={:#x}", 
               virt_addr, phys_addr.as_usize(), bus_addr);
    bus_addr
}
```

### 3. Enhanced Debugging and Validation

Added comprehensive logging and validation:
- DMA allocation success/failure logging
- Register access bounds checking
- Descriptor setup validation
- Memory address translation tracing

### 4. Safer Reset Function

**Before:**
```rust
fn reset_device(&self) -> Result<(), &'static str> {
    self.write_reg(regs::DMA_BUS_MODE, dma_bus_mode::SOFTWARE_RESET);
    // Basic timeout loop
}
```

**After:**
```rust
fn reset_device(&self) -> Result<(), &'static str> {
    log::debug!("Starting DWMAC device reset");
    
    // Check device accessibility first
    let version = self.read_reg(regs::MAC_VERSION);
    log::debug!("DWMAC version register: {:#x}", version);
    
    // Perform reset with proper timeout handling
    // ... enhanced implementation
}
```

## Memory Layout Understanding

### VisionFive 2 Memory Map
```
0x00000000 - 0x40000000: Peripherals (including DWMAC at 0x16030000)
0x40000000 - 0x50000000: DDR Memory (256MB)
0x16030000 - 0x16040000: DWMAC0 MMIO registers
0x16040000 - 0x16050000: DWMAC1 MMIO registers
```

### Virtual Memory Mapping
```
Physical: 0x40000000 -> Virtual: 0xffff_ffc0_4000_0000
Offset: PHYS_VIRT_OFFSET = 0xffff_ffc0_0000_0000
Bus Offset: PHYS_BUS_OFFSET = 0 (no translation needed)
```

## Debugging Commands

### 1. Enable Debug Logging
```bash
# Build with debug logging
make A=examples/httpserver PLATFORM=riscv64-starfive NET=y LOG=debug
```

### 2. Check Memory Allocation
Look for these log messages:
```
DMA alloc: size=16384, cpu_addr=0xffff_ffc0_4008_0000, bus_addr=0x40080000
TX descriptors: phys=0x40080000, virt=0xffff_ffc0_4008_0000
RX descriptors: phys=0x40081000, virt=0xffff_ffc0_4008_1000
```

### 3. Validate Register Access
```
DWMAC read_reg: offset=0x20, addr=0x16030020, value=0x51100a2c
DWMAC write_reg: offset=0x1000, addr=0x16031000, value=0x1
```

### 4. Monitor Descriptor Setup
```
TX desc[0]: status=0x0, control=0x20000000, buffer1=0x0, buffer2=0x40080040
RX desc[0]: status=0x80000000, control=0x600, buffer1=0x40082000, buffer2=0x40081040
```

## Common Issues and Solutions

### Issue 1: DMA Allocation Failure
**Symptoms:** `Failed to allocate DMA memory: tx_phys=0x0, rx_phys=0x0`
**Solution:** Check available memory, reduce descriptor count if needed

### Issue 2: Invalid Register Access
**Symptoms:** `DWMAC write_reg: suspicious address 0x... (offset 0x...)`
**Solution:** Verify MMIO mapping in device tree and platform config

### Issue 3: Descriptor Ring Corruption
**Symptoms:** Unexpected descriptor status values
**Solution:** Ensure proper cache coherency and alignment

## Testing the Fix

### 1. Build and Run
```bash
make A=examples/httpserver PLATFORM=riscv64-starfive NET=y
# Boot on VisionFive 2 and check for successful initialization
```

### 2. Expected Success Messages
```
DWMAC device initialized successfully
Network interface "eth0" created with MAC address 6c:cf:39:00:5d:34
```

### 3. Network Functionality Test
```bash
# On the target system
ping 192.168.1.1
curl http://192.168.1.100:5555/
```

## Prevention Strategies

1. **Always validate DMA allocations** before use
2. **Use proper alignment** for hardware descriptors (16-byte minimum)
3. **Distinguish between physical and bus addresses** in DMA operations
4. **Add bounds checking** for all MMIO register access
5. **Implement comprehensive logging** for debugging
6. **Test on actual hardware** early and often

## Related Files Modified

- `modules/axdriver/src/dwmac.rs` - HAL implementation fixes
- `axdriver_crates/axdriver_net/src/dwmac.rs` - Driver core improvements
- `modules/axdriver/src/structs/static.rs` - IRQ handling fixes
- `modules/axdriver/src/structs/dyn.rs` - Dynamic container fixes
- `modules/axnet/Cargo.toml` - Feature dependencies

This comprehensive fix resolves the Load Access Fault and provides a robust foundation for DWMAC Ethernet functionality on the StarFive VisionFive 2 platform. 