# 开发日志

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