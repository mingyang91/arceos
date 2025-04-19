# MMIO Async Example

This example demonstrates how to use memory-mapped I/O (MMIO) with asynchronous programming in ArceOS.

## Features

- Demonstrates registering an MMIO device with interrupt handling
- Shows how to create and use asynchronous MMIO events
- Uses async/await pattern for handling MMIO operations

## How it works

1. The example registers a simulated MMIO device that triggers interrupts
2. It then creates an async task that waits for the MMIO event
3. When the interrupt occurs, the async task is woken up and continues execution

## Usage

```bash
make A=examples/mmio_async ARCH=<arch> LOG=info run
```

Where `<arch>` is one of `x86_64`, `riscv64`, `aarch64`, or `loongarch64`. 