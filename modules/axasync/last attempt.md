```bash
[  1.450743 0 axdriver_net::dwmac:758] 🔍 MAC CONFIG(0x0): 0x1f8203
[  1.451979 0 axdriver_net::dwmac:758] 🔍 PHYIF_CONTROL_STATUS(0xf8): 0xb0000
[  1.453357 0 axdriver_net::dwmac:758] 🔍 GMAC_DEBUG_STATUS(0x114): 0x0
[  1.454651 0 axdriver_net::dwmac:758] 🔍 DMA_STATUS(0x1008): 0x0
[  1.455859 0 axdriver_net::dwmac:758] 🔍 DMA_DEBUG_STATUS0(0x100c): 0x6300
[  1.457209 0 axdriver_net::dwmac:758] 🔍 DMA_DEBUG_STATUS1(0x1010): 0x0
[  1.458517 0 axdriver_net::dwmac:758] 🔍 DMA_DEBUG_STATUS2(0x1014): 0x0
[  1.459825 0 axdriver_net::dwmac:758] 🔍 DMA_CHAN_CUR_TX_DESC(0x1144): 0x4027b7d0
[  1.461274 0 axdriver_net::dwmac:758] 🔍 DMA_CHAN_CUR_RX_DESC(0x114c): 0x4027b920
[  1.462724 0 axdriver_net::dwmac:758] 🔍 DMA_CHAN_STATUS(0x1160): 0x44
[  1.464017 0 axdriver_net::dwmac:826] DMA_CHAN0_DEBUG_STATUS: 0x44, tx_fsm: 0x4, rx_fsm: 0x0
[  1.465595 0 axdriver_net::dwmac:758] 🔍 DMA_CHAN_RX_CTRL(0x1108): 0x80001
[  1.466945 0 axdriver_net::dwmac:758] 🔍 MTL TXQ0_OPERATION_MODE(0xd00): 0x70008
[  1.468381 0 axdriver_net::dwmac:758] 🔍 MTL TXQ0_DEBUG(0xd08): 0x0
[  1.469632 0 axdriver_net::dwmac:758] 🔍 MTL TXQ0_QUANTUM_WEIGHT(0xd18): 0x0
[  1.471010 0 axdriver_net::dwmac:758] 🔍 MTL RXQ0_OPERATION_MODE(0xd30): 0x700020
[  1.472460 0 axdriver_net::dwmac:758] 🔍 MTL RXQ0_DEBUG(0xd38): 0x10010
[  1.473768 0 axdriver_net::dwmac:847] RX buffer owned by CPU, index: 0
[  1.475033 0 axdriver_net::dwmac:847] RX buffer owned by CPU, index: 1
[  1.476298 0 axdriver_net::dwmac:940] Packet received, length: 150, RX index: 1
[  1.477691 0 axnet::smoltcp_impl:259] RECV 150 bytes: [01, 00, 5E, 00, 00, FB, DE, 5D, F8, 9D, 61, C9, 08, 00, 45, 00, 00, 84, AB, 0D, 00, 00, FF, 11, 6D, A6, C0, A8, 01, 11, E0, 00, 00, FB, 14, E9, 14, E9, 00, 70, 9A, 11, 00, 00, 00, 00, 00, 05, 00, 00, 00, 00, 00, 00, 0F, 5F, 63, 6F, 6D, 70, 61, 6E, 69, 6F, 6E, 2D, 6C, 69, 6E, 6B, 04, 5F, 74, 63, 70, 05, 6C, 6F, 63, 61, 6C, 00, 00, 0C, 80, 01, 07, 5F, 72, 64, 6C, 69, 6E, 6B, C0, 1C, 00, 0C, 80, 01, 04, 5F, 68, 61, 70, C0, 1C, 00, 0C, 80, 01, 04, 5F, 68, 61, 70, 04, 5F, 75, 64, 70, C0, 21, 00, 0C, 80, 01, 0C, 5F, 73, 6C, 65, 65, 70, 2D, 70, 72, 6F, 78, 79, C0, 4A, 00, 0C, 80, 01, 83, 06, 3B, 39]
[  1.487257 0 axdriver_net::dwmac:963] RX buffer recycled, RX index: 0
[  1.488507 0 axdriver_net::dwmac:847] RX buffer owned by CPU, index: 1
[  1.489772 0 axdriver_net::dwmac:940] Packet received, length: 170, RX index: 2
[  1.491165 0 axnet::smoltcp_impl:259] RECV 170 bytes: [33, 33, 00, 00, 00, FB, DE, 5D, F8, 9D, 61, C9, 86, DD, 60, 0F, 07, 00, 00, 70, 11, FF, FE, 80, 00, 00, 00, 00, 00, 00, 10, 88, B3, F6, E0, 4E, 29, 56, FF, 02, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, 00, FB, 14, E9, 14, E9, 00, 70, 70, 24, 00, 00, 00, 00, 00, 05, 00, 00, 00, 00, 00, 00, 0F, 5F, 63, 6F, 6D, 70, 61, 6E, 69, 6F, 6E, 2D, 6C, 69, 6E, 6B, 04, 5F, 74, 63, 70, 05, 6C, 6F, 63, 61, 6C, 00, 00, 0C, 80, 01, 07, 5F, 72, 64, 6C, 69, 6E, 6B, C0, 1C, 00, 0C, 80, 01, 04, 5F, 68, 61, 70, C0, 1C, 00, 0C, 80, 01, 04, 5F, 68, 61, 70, 04, 5F, 75, 64, 70, C0, 21, 00, 0C, 80, 01, 0C, 5F, 73, 6C, 65, 65, 70, 2D, 70, 72, 6F, 78, 79, C0, 4A, 00, 0C, 80, 01, 80, E1, 1B, D1]
[  1.501868 0 axdriver_net::dwmac:963] RX buffer recycled, RX index: 1
```