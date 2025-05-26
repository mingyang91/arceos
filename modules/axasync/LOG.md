# 开发日志

## 2025-05-26

内核启动成功，日志已输出，~~目前内核启动后会`panic`在`allocator`~~，不管怎么说，内核启动成功了。
物理内存设置过大会导致allocator panic，问题未知。目前确认`0x1000_0000`不会panic，但是`0x8000_0000`以上会panic。但星光2的物理内存为4GB，所以应当支持大约 `0x1_0000_0000 - <reserved>` 内存空间。
详见[2025-05-26.md](./2025-05-26.md)

```
U-Boot SPL 2021.10 (Feb 28 2023 - 21:44:53 +0800)
DDR version: dc2e84f0.
Trying to boot from SPI

OpenSBI v1.2
   ____                    _____ ____ _____
  / __ \                  / ____|  _ \_   _|
 | |  | |_ __   ___ _ __ | (___ | |_) || |
 | |  | | '_ \ / _ \ '_ \ \___ \|  _ < | |
 | |__| | |_) |  __/ | | |____) | |_) || |_
  \____/| .__/ \___|_| |_|_____/|____/_____|
        | |
        |_|

Platform Name             : StarFive VisionFive V2
Platform Features         : medeleg
Platform HART Count       : 5
Platform IPI Device       : aclint-mswi
Platform Timer Device     : aclint-mtimer @ 4000000Hz
Platform Console Device   : uart8250
Platform HSM Device       : jh7110-hsm
Platform PMU Device       : ---
Platform Reboot Device    : pm-reset
Platform Shutdown Device  : pm-reset
Firmware Base             : 0x40000000
Firmware Size             : 292 KB
Runtime SBI Version       : 1.0

Domain0 Name              : root
Domain0 Boot HART         : 1
Domain0 HARTs             : 0*,1*,2*,3*,4*
Domain0 Region00          : 0x0000000002000000-0x000000000200ffff (I)
Domain0 Region01          : 0x0000000040000000-0x000000004007ffff ()
Domain0 Region02          : 0x0000000000000000-0xffffffffffffffff (R,W,X)
Domain0 Next Address      : 0x0000000040200000
Domain0 Next Arg1         : 0x0000000042200000
Domain0 Next Mode         : S-mode
Domain0 SysReset          : yes

Boot HART ID              : 1
Boot HART Domain          : root
Boot HART Priv Version    : v1.11
Boot HART Base ISA        : rv64imafdcbx
Boot HART ISA Extensions  : none
Boot HART PMP Count       : 8
Boot HART PMP Granularity : 4096
Boot HART PMP Address Bits: 34
Boot HART MHPM Count      : 2
Boot HART MIDELEG         : 0x0000000000000222
Boot HART MEDELEG         : 0x000000000000b109


U-Boot 2021.10 (Feb 28 2023 - 21:44:53 +0800), Build: jenkins-VF2_515_Branch_SDK_Release-31

CPU:   rv64imacu
Model: StarFive VisionFive V2
DRAM:  4 GiB
MMC:   sdio0@16010000: 0, sdio1@16020000: 1
Loading Environment from SPIFlash... SF: Detected gd25lq128 with page size 256 Bytes, erase size 4 KiB, total 16 MiB
*** Warning - bad CRC, using default environment

StarFive EEPROM format v2

--------EEPROM INFO--------
Vendor : StarFive Technology Co., Ltd.
Product full SN: VF7110B1-2310-D004E000-00003171
data version: 0x2
PCB revision: 0xb2
BOM revision: A
Ethernet MAC0 address: 6c:cf:39:00:5d:34
Ethernet MAC1 address: 6c:cf:39:00:5d:35
--------EEPROM INFO--------

In:    serial@10000000
Out:   serial@10000000
Err:   serial@10000000
Model: StarFive VisionFive V2
Net:   eth0: ethernet@16030000, eth1: ethernet@16040000
switch to partitions #0, OK
mmc1 is current device
found device 1
bootmode flash device 1
64 bytes read in 5 ms (11.7 KiB/s)
Importing environment from mmc1 ...
Can't set block device
Hit any key to stop autoboot:  0 
158539 bytes read in 16 ms (9.4 MiB/s)
## Loading kernel from FIT Image at c0000000 ...
   Using 'conf' configuration
   Trying 'kernel' kernel subimage
     Description:  Linux kernel for zCore-visionfive
     Type:         Kernel Image
     Compression:  gzip compressed
     Data Start:   0xc00000f0
     Data Size:    103935 Bytes = 101.5 KiB
     Architecture: RISC-V
     OS:           Linux
     Load Address: 0x40200000
     Entry Point:  0x40200000
   Verifying Hash Integrity ... OK
## Loading fdt from FIT Image at c0000000 ...
   Using 'conf' configuration
   Trying 'fdt' fdt subimage
     Description:  Flattened Device Tree blob for zCore-visionfive
     Type:         Flat Device Tree
     Compression:  uncompressed
     Data Start:   0xc00197b4
     Data Size:    52853 Bytes = 51.6 KiB
     Architecture: RISC-V
   Verifying Hash Integrity ... OK
   Booting using the fdt blob at 0xc00197b4
   Uncompressing Kernel Image
   Using Device Tree in place at 00000000c00197b4, end 00000000c0029628

Starting kernel ...

clk u5_dw_i2c_clk_core already disabled
clk u5_dw_i2c_clk_apb already disabled

       d8888                            .d88888b.   .d8888b.
      d88888                           d88P" "Y88b d88P  Y88b
     d88P888                           888     888 Y88b.
    d88P 888 888d888  .d8888b  .d88b.  888     888  "Y888b.
   d88P  888 888P"   d88P"    d8P  Y8b 888     888     "Y88b.
  d88P   888 888     888      88888888 888     888       "888
 d8888888888 888     Y88b.    Y8b.     Y88b. .d88P Y88b  d88P
d88P     888 888      "Y8888P  "Y8888   "Y88888P"   "Y8888P"

arch = riscv64
platform = riscv64-starfive
target = riscv64gc-unknown-none-elf
build_mode = release
log_level = info
smp = 4

[  5.103254 axruntime:130] Logging is enabled.
[  5.108800 axruntime:131] Primary CPU 1 started, dtb = 0xc00197b4.
[  5.116166 axruntime:133] Found physcial memory regions:
[  5.122667 axruntime:135]   [PA:0x40200000, PA:0x40222000) .text (READ | EXECUTE | RESERVED)
[  5.132286 axruntime:135]   [PA:0x40222000, PA:0x4022d000) .rodata (READ | RESERVED)
[  5.141213 axruntime:135]   [PA:0x4022d000, PA:0x40230000) .data .tdata .tbss .percpu (READ | WRITE | RESERVED)
[  5.152480 axruntime:135]   [PA:0x40230000, PA:0x40330000) boot stack (READ | WRITE | RESERVED)
[  5.162360 axruntime:135]   [PA:0x40330000, PA:0x40338000) .bss (READ | WRITE | RESERVED)
[  5.171719 axruntime:135]   [PA:0x40338000, PA:0x50000000) free memory (READ | WRITE | FREE)
[  5.181340 axruntime:135]   [PA:0xc000000, PA:0x10000000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.191393 axruntime:135]   [PA:0x10000000, PA:0x10001000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.201533 axruntime:135]   [PA:0x10010000, PA:0x10011000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.211673 axruntime:135]   [PA:0x10020000, PA:0x10021000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.221813 axruntime:135]   [PA:0x10030000, PA:0x10031000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.231953 axruntime:135]   [PA:0x10040000, PA:0x10041000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.242093 axruntime:135]   [PA:0x10050000, PA:0x10051000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.252233 axruntime:135]   [PA:0x16010000, PA:0x16011000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.262373 axruntime:135]   [PA:0x16020000, PA:0x16021000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.272513 axruntime:135]   [PA:0x16030000, PA:0x16031000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.282653 axruntime:135]   [PA:0x16040000, PA:0x16041000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.292793 axruntime:135]   [PA:0x13040000, PA:0x13041000) mmio (READ | WRITE | DEVICE | RESERVED)
[  5.302933 axruntime:213] Initialize global memory allocator...
[  5.310040 axruntime:214]   use TLSF allocator.
[  5.315872 axmm:72] Initialize virtual memory management...
[  5.329465 axruntime:150] Initialize platform devices...
[  5.335806 axdriver:152] Initialize device drivers...
[  5.342043 axdriver:153]   device model: static
[  5.347763 axdriver::bus::mmio:6] probing bus devices...
[  5.354265 axhal::arch::riscv::trap:24] No registered handler for trap PAGE_FAULT
[  5.362930 axruntime::lang_items:5] panicked at modules/axhal/src/arch/riscv/trap.rs:25:9:
Unhandled Supervisor Page Fault @ 0xffffffc040205b10, fault_vaddr=VA:0xffffffc082001000 (READ):
TrapFrame {
    regs: GeneralRegisters {
        ra: 0xffffffc040205a06,
        sp: 0xffffffc040266ba0,
        gp: 0xffffffc04021a2b8,
        tp: 0xffffffc040266b58,
        t0: 0x5a00000001,
        t1: 0x30000,
        t2: 0xe00000001,
        s0: 0xffffffc04026ac48,
        s1: 0x2,
        a0: 0xffffffc040224aec,
        a1: 0xffffffc04026da6f,
        a2: 0xffffffffffffffff,
        a3: 0x2,
        a4: 0xc0,
        a5: 0xf0f0f0f,
        a6: 0x74727000,
        a7: 0x30010000,
        s2: 0x74726976,
        s3: 0x1000,
        s4: 0xffffffc082001000,
        s5: 0xffffffc040224668,
        s6: 0x20,
        s7: 0xffffffc0403375b8,
        s8: 0x82001000,
        s9: 0x1,
        s10: 0xffffffc04026d25c,
        s11: 0x1000,
        t3: 0x55555000,
        t4: 0x33333000,
        t5: 0xffffffc04026d270,
        t6: 0xf,
    },
    sepc: 0xffffffc040205b10,
    sstatus: 0x8000000200006100,
}
[  5.463203 axhal::platform::riscv64_starfive::misc:3] Shutting down...
i2c read: write daddr 36 to
i2c read: write daddr 36 to
i2c read: write daddr 36 to
i2c read: write daddr 36 to
i2c read: write daddr 36 to
i2c read: write daddr 36 to
i2c read: write daddr 36 to
i2c read: write daddr 36 to
i2c read: write daddr 36 to
i2c read: write daddr 36 to
cannot read pmic power register

```

## 2025-05-25
成功烧录到 SD Card 中并启动板子，但内核启动失败，且未能从UART观察到任何日志输出。
详见[2025-05-25.md](./2025-05-25.md)

## 2025-05-24
### 编译 async_server 示例代码
```
make A=examples/async_server ARCH=riscv64 PLATFORM=riscv64-starfive LOG=info NET=y SMP=4 BUS=mmio APP_FEATURES=starfive build
```

### 调试器接线
#### lichee pi 4a
- u0-rx -> 调试器 TX
- u0-tx -> 调试器 RX


#### starfive jh7110
拨码启动模式可以继续使用默认的 1-bit QSPI Nor Flash（Low&Low)
串口调试器接线
- 6 GND -> 调试器 GND
- 8 UART TX -> 调试器 RX
- 10 UART RX -> 调试器 TX

### 串口可以显示系统启动日志
```
λ python -m serial.tools.miniterm --eol LF --dtr 0 --rts 0 --filter direct /dev/tty.usbserial-21301 115200
--- forcing DTR inactive
--- forcing RTS inactive
--- Miniterm on /dev/tty.usbserial-21301  115200,8,N,1 ---
--- Quit: Ctrl+] | Menu: Ctrl+T | Help: Ctrl+T followed by Ctrl+H ---
�
U-Boot SPL 2021.10 (Feb 28 2023 - 21:44:53 +0800)
DDR version: dc2e84f0.
Trying to boot from SPI

OpenSBI v1.2
   ____                    _____ ____ _____
  / __ \                  / ____|  _ \_   _|
 | |  | |_ __   ___ _ __ | (___ | |_) || |
 | |  | | '_ \ / _ \ '_ \ \___ \|  _ < | |
 | |__| | |_) |  __/ | | |____) | |_) || |_
  \____/| .__/ \___|_| |_|_____/|____/_____|
        | |
        |_|

Platform Name             : StarFive VisionFive V2
Platform Features         : medeleg
Platform HART Count       : 5
Platform IPI Device       : aclint-mswi
Platform Timer Device     : aclint-mtimer @ 4000000Hz
Platform Console Device   : uart8250
Platform HSM Device       : jh7110-hsm
Platform PMU Device       : ---
Platform Reboot Device    : pm-reset
Platform Shutdown Device  : pm-reset
Firmware Base             : 0x40000000
Firmware Size             : 292 KB
Runtime SBI Version       : 1.0

Domain0 Name              : root
Domain0 Boot HART         : 1
Domain0 HARTs             : 0*,1*,2*,3*,4*
Domain0 Region00          : 0x0000000002000000-0x000000000200ffff (I)
Domain0 Region01          : 0x0000000040000000-0x000000004007ffff ()
Domain0 Region02          : 0x0000000000000000-0xffffffffffffffff (R,W,X)
Domain0 Next Address      : 0x0000000040200000
Domain0 Next Arg1         : 0x0000000042200000
Domain0 Next Mode         : S-mode
Domain0 SysReset          : yes

Boot HART ID              : 1
Boot HART Domain          : root
Boot HART Priv Version    : v1.11
Boot HART Base ISA        : rv64imafdcbx
Boot HART ISA Extensions  : none
Boot HART PMP Count       : 8
Boot HART PMP Granularity : 4096
Boot HART PMP Address Bits: 34
Boot HART MHPM Count      : 2
Boot HART MIDELEG         : 0x0000000000000222
Boot HART MEDELEG         : 0x000000000000b109


U-Boot 2021.10 (Feb 28 2023 - 21:44:53 +0800), Build: jenkins-VF2_515_Branch_SDK_Release-31

CPU:   rv64imacu
Model: StarFive VisionFive V2
DRAM:  4 GiB
MMC:   sdio0@16010000: 0, sdio1@16020000: 1
Loading Environment from SPIFlash... SF: Detected gd25lq128 with page size 256 Bytes, erase size 4 KiB, total 16 MiB
*** Warning - bad CRC, using default environment

StarFive EEPROM format v2

--------EEPROM INFO--------
Vendor : StarFive Technology Co., Ltd.
Product full SN: VF7110B1-2310-D004E000-00003171
data version: 0x2
PCB revision: 0xb2
BOM revision: A
Ethernet MAC0 address: 6c:cf:39:00:5d:34
Ethernet MAC1 address: 6c:cf:39:00:5d:35
--------EEPROM INFO--------

In:    serial@10000000
Out:   serial@10000000
Err:   serial@10000000
Model: StarFive VisionFive V2
Net:   eth0: ethernet@16030000, eth1: ethernet@16040000
switch to partitions #0, OK
mmc1 is current device
found device 1
bootmode flash device 1
** No partition table - mmc 1 **
Couldn't find partition mmc 1:3
Can't set block device
** No partition table - mmc 1 **
Couldn't find partition mmc 1:3
Can't set block device
Hit any key to stop autoboot:  0 
** No partition table - mmc 1 **
Couldn't find partition mmc 1:3
Can't set block device
Importing environment from mmc1 ...
## Info: input data size = 6210 = 0x1842
** No partition table - mmc 1 **
Couldn't find partition mmc 1:3
Can't set block device
## Warning: defaulting to text format
## Error: "boot2" not defined
switch to partitions #0, OK
mmc1 is current device
** No partition table - mmc 1 **
Couldn't find partition mmc 1:1
ethernet@16030000 Waiting for PHY auto negotiation to complete......... TIMEOUT !
phy_startup() failed: -110FAILED: -110ethernet@16040000 Waiting for PHY auto negotiation to complete......... TIMEOUT !
phy_startup() failed: -110FAILED: -110ethernet@16030000 Waiting for PHY auto negotiation to complete......... TIMEOUT !
phy_startup() failed: -110FAILED: -110ethernet@16040000 Waiting for PHY auto negotiation to complete......... TIMEOUT !
phy_startup() failed: -110FAILED: -110StarFive # 
--- exit ---
```

## 2025-05-17
已通过 async_server 示例代码测试
```
λ wrk -t12 -c36 -d30s http://127.0.0.1:5555/
Running 30s test @ http://127.0.0.1:5555/
  12 threads and 36 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   674.44us    1.43ms  11.15ms   94.00%
    Req/Sec   208.37    358.97     1.84k    93.33%
  650 requests in 30.07s, 271.68KB read
  Socket errors: connect 0, read 306, write 21, timeout 0
Requests/sec:     21.61
Transfer/sec:      9.03KB
```
- 增加 spawn 与 spawn_local 函数，用于并行执行任务
- 将 axasync 模块中的 block_on 与 poll_once 函数移至 executor 模块中，并添加 dummy_waker 函数。
- 将 axasync 模块中的 init 与 shutdown 函数移至 lib.rs 中，并添加 executor 模块中的 block_on 与 dummy_waker 函数。

## 2025-05-12
真 - 异步版 axnet 实现，支持 async_client 与 async_server 示例代码

## 2025-05-11
学习 dtb/dts 机制，深入理解 IRQ number 与 MMIO 地址的映射关系，思考从何入手将中断与设备注册关联起来。

## 2025-05-10
成功在 qemu 中复现硬件中断代码，目前硬件中断发生时已经可以出发点中断打印函数。

## 2025-04-19
尝试从 mmio 层面支持 async 功能，尚未成功. 发现需要从 IRQ 和 PLIC 层面硬件中断入手。

## 2025-04-16
之前实现的 axnet 异步功能不够彻底，将其替换为返回 Future 数据结构的 API 实现，同时平行支持同步 API

## 2025-04-09
从 smoltcp 层面上支持 async 功能, 并添加了 async_client 与 async_server 示例代码

## 2025-04-08
添加 axasync 模块和 async_mode 示例代码，实现基础 executor 功能, 并添加了 Sleep 功能和更多同步 API
尝试为 axnet 加入 async 支持