# ArceOS Async Runtime (axasync)

A lightweight asynchronous runtime for ArceOS, inspired by the design of [embassy-rs](https://github.com/embassy-rs/embassy).

## Features

- Async/await support for ArceOS applications
- Lightweight task management
- Support for timeouts and sleep operations
- Minimal overhead executor
- Compatible with the standard Rust Future trait

## Design

The axasync module provides several key components:

1. **Futures and Async Primitives**:
   - Implements core async operations like `sleep` and timeout
   - Uses Rust's standard Future trait for compatibility

2. **Waker Implementations**:
   - Provides waking mechanisms for async tasks 
   - Timer-based wakers for scheduling future work
   - Task-based wakers that integrate with ArceOS's task system

3. **Task Executor**:
   - Multi-threaded executor to run async tasks
   - Efficient scheduling of futures
   - Support for joining on task completion via JoinHandle

## Usage

To use the axasync module, enable the appropriate features in your application's Cargo.toml:

```toml
[dependencies]
axasync = { path = "modules/axasync", features = ["multitask", "timer"] }
```

### Basic Example

```rust
use axasync::{block_on, time::sleep};
use core::time::Duration;

fn main() {
    block_on(async {
        println!("Hello from async!");
        sleep(Duration::from_millis(1000)).await;
        println!("Woke up after 1 second");
    });
}
```

### Using Timeouts

```rust
use axasync::{time::TimeoutExt, time::sleep, block_on};
use core::time::Duration;

async fn long_operation() -> u32 {
    sleep(Duration::from_secs(10)).await;
    42
}

fn main() {
    block_on(async {
        match long_operation().timeout(Duration::from_secs(1)).await {
            Ok(value) => println!("Operation completed with value: {}", value),
            Err(_) => println!("Operation timed out"),
        }
    });
}
```

### Spawning Tasks (with multitask feature)

```rust
use axasync::{Executor, time::sleep};
use core::time::Duration;

fn main() {
    let executor = Executor::new();
    
    let handle = executor.spawn(async {
        sleep(Duration::from_millis(100)).await;
        println!("Task 1 completed");
        42
    });
    
    let handle2 = executor.spawn(async {
        sleep(Duration::from_millis(200)).await;
        println!("Task 2 completed");
        "Hello"
    });
    
    // Run the executor until all tasks complete
    executor.run();
    
    // Alternatively, run it step by step
    // while executor.step() {}
}
```

## Integration with ArceOS

Axasync is designed to work seamlessly with ArceOS:

### Timer Integration

When using the `timer` feature, the axasync module integrates with ArceOS's timer interrupt system to efficiently wake up futures at specified times:

1. The module registers a timer event handler with axruntime
2. On each timer tick, pending timer events are checked
3. Any expired timers trigger the appropriate waker

To enable timer integration in axruntime, add the following feature to your application:

```toml
[dependencies.axruntime]
features = ["axasync-timer"]
```

### Task System Integration

With the `multitask` feature enabled, axasync integrates with axtask to:

1. Use the existing task system for async task scheduling
2. Provide wakers that properly unblock tasks
3. Efficiently manage task states for async operations

## Configuration

The axasync module can be configured with the following Cargo features:

- `multitask`: Enable multi-task support (requires axtask)
- `irq`: Enable interrupt handling support
- `timer`: Enable timer functionality for timeouts and sleep operations 


## Utils
run async server example:
```bash
make A=examples/async_server ARCH=riscv64 PLATFORM=riscv64-qemu-virt LOG=debug NET=y SMP=1 BUS=mmio FEATURES=net,bus-mmio APP_FEATURES=default run
```

curl:
```bash
curl http://127.0.0.1:5555 -v
```

wrk:
```bash
wrk -d10s -c16 http://127.0.0.1:5555/
```
