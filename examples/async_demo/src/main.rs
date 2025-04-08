#![no_std]
#![no_main]

use axstd::println;
use axstd::time::Duration;
use core::result::Result::{Err, Ok};

use axasync::{block_on, sleep, TimeoutExt};

#[cfg(feature = "multitask")]
use axasync::Executor;

// A simple async function that sleeps for a while
async fn sleep_and_print(id: u32, duration_ms: u64) -> u32 {
    println!("Task {} started", id);
    sleep(Duration::from_millis(duration_ms)).await;
    println!("Task {} completed after {}ms", id, duration_ms);
    id
}

// An async function with a timeout
async fn demo_timeout() {
    println!("Starting timeout demo");

    let result = sleep_and_print(99, 2000)
        .timeout(Duration::from_millis(500))
        .await;

    match result {
        Ok(id) => println!("Task {} completed within timeout", id),
        Err(_) => println!("Task timed out"),
    }

    println!("Timeout demo completed");
}

#[no_mangle]
fn main() {
    println!("Async Demo: Hello from ArceOS!");

    // Basic block_on demo
    block_on(async {
        println!("Running a simple async task");
        sleep(Duration::from_millis(100)).await;
        println!("Simple async task completed");
    });

    // Timeout demo
    block_on(demo_timeout());

    // Executor demo for multi-tasking
    #[cfg(feature = "multitask")]
    {
        println!("\nStarting executor demo");
        let executor = Executor::new();

        // Spawn multiple tasks
        let handle1 = executor.spawn(sleep_and_print(1, 500));
        let handle2 = executor.spawn(sleep_and_print(2, 1000));
        let handle3 = executor.spawn(sleep_and_print(3, 200));

        // Run all tasks to completion
        executor.run();

        println!("All tasks completed in the executor");
    }

    println!("Async Demo: Done!");
}
