# 开发日志

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