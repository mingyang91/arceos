# DWMAC Driver Implementation Guide

## Required Implementation in axdriver_net crate

The DWMAC driver needs to be implemented in the `axdriver_net` crate. Here's the structure:

### 1. Driver Trait Implementation

```rust
// In axdriver_net/src/dwmac/mod.rs

use axdriver_base::{BaseDriverOps, DevError, DevResult, DeviceType};
use core::ptr::NonNull;

pub trait DwmacHal: Send + Sync {
    fn dma_alloc(size: usize) -> (PhysAddr, NonNull<u8>);
    unsafe fn dma_dealloc(paddr: PhysAddr, vaddr: NonNull<u8>, size: usize) -> i32;
    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, size: usize) -> NonNull<u8>;
    unsafe fn mmio_virt_to_phys(vaddr: NonNull<u8>, size: usize) -> PhysAddr;
    fn wait_until(duration: core::time::Duration) -> Result<(), &'static str>;
}

pub type PhysAddr = usize;

pub struct DwmacNic<H: DwmacHal> {
    base_addr: NonNull<u8>,
    mac_addr: [u8; 6],
    _phantom: core::marker::PhantomData<H>,
}

impl<H: DwmacHal> DwmacNic<H> {
    pub fn init(base_addr: NonNull<u8>, _size: usize) -> Result<Self, &'static str> {
        // Initialize DWMAC hardware
        // 1. Reset the device
        // 2. Configure DMA
        // 3. Set up MAC
        // 4. Configure PHY
        // 5. Read MAC address from hardware or generate one
        
        let mac_addr = [0x6c, 0xcf, 0x39, 0x00, 0x5d, 0x34]; // From VisionFive 2 EEPROM
        
        Ok(Self {
            base_addr,
            mac_addr,
            _phantom: core::marker::PhantomData,
        })
    }
}

impl<H: DwmacHal> BaseDriverOps for DwmacNic<H> {
    fn device_type(&self) -> DeviceType {
        DeviceType::Net
    }

    fn device_name(&self) -> &str {
        "dwmac-ethernet"
    }
}

impl<H: DwmacHal> crate::NetDriverOps for DwmacNic<H> {
    fn mac_address(&self) -> crate::EthernetAddress {
        crate::EthernetAddress(self.mac_addr)
    }

    fn can_transmit(&self) -> bool {
        // Check if TX queue has space
        true
    }

    fn can_receive(&self) -> bool {
        // Check if RX queue has packets
        false
    }

    fn rx_queue_size(&self) -> usize {
        256 // Configurable
    }

    fn tx_queue_size(&self) -> usize {
        256 // Configurable
    }

    fn recycle_rx_buffer(&mut self, _buf: crate::NetBufPtr) -> DevResult {
        // Return RX buffer to hardware
        Ok(())
    }

    fn recycle_tx_buffers(&mut self) -> DevResult {
        // Reclaim completed TX buffers
        Ok(())
    }

    fn transmit(&mut self, _buf: crate::NetBufPtr) -> DevResult {
        // Submit packet for transmission
        Err(DevError::Unsupported)
    }

    fn receive(&mut self) -> DevResult<crate::NetBufPtr> {
        // Receive packet from hardware
        Err(DevError::Again)
    }

    fn alloc_tx_buffer(&mut self, size: usize) -> DevResult<crate::NetBufPtr> {
        // Allocate buffer for transmission
        crate::NetBufPool::alloc_tx_buffer(size)
    }
}
```

### 2. Hardware Register Definitions

```rust
// DWMAC register offsets (Synopsys DesignWare MAC v5.10a)
const MAC_CONFIG: usize = 0x0000;
const MAC_FRAME_FILTER: usize = 0x0004;
const MAC_HASH_TABLE_REG0: usize = 0x0008;
const MAC_HASH_TABLE_REG1: usize = 0x000C;
const MAC_GMII_ADDRESS: usize = 0x0010;
const MAC_GMII_DATA: usize = 0x0014;
const MAC_FLOW_CONTROL: usize = 0x0018;
const MAC_VLAN_TAG: usize = 0x001C;
const MAC_VERSION: usize = 0x0020;
const MAC_DEBUG: usize = 0x0024;
const MAC_REMOTE_WAKE_FILTER: usize = 0x0028;
const MAC_PMT_CONTROL_STATUS: usize = 0x002C;

// DMA register offsets
const DMA_BUS_MODE: usize = 0x1000;
const DMA_TX_POLL_DEMAND: usize = 0x1004;
const DMA_RX_POLL_DEMAND: usize = 0x1008;
const DMA_RX_DESCRIPTOR_LIST: usize = 0x100C;
const DMA_TX_DESCRIPTOR_LIST: usize = 0x1010;
const DMA_STATUS: usize = 0x1014;
const DMA_OPERATION_MODE: usize = 0x1018;
const DMA_INTERRUPT_ENABLE: usize = 0x101C;
```

### 3. Key Implementation Details

1. **PHY Management**: Use MDIO interface to configure the RGMII PHY
2. **DMA Setup**: Configure descriptor rings for TX/RX
3. **MAC Configuration**: Set up frame filtering, flow control
4. **Interrupt Handling**: Handle TX/RX completion interrupts
5. **Clock Management**: Configure required clocks from device tree

## VisionFive 2 Specific Configuration

- **PHY Mode**: RGMII-ID (with internal delay)
- **MAC Addresses**: 
  - eth0: 6c:cf:39:00:5d:34
  - eth1: 6c:cf:39:00:5d:35
- **Base Addresses**:
  - DWMAC0: 0x16030000
  - DWMAC1: 0x16040000
- **IRQs**: 7, 6, 5 for DWMAC0; 78, 77, 76 for DWMAC1 