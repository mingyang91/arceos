//! Demonstration of async synchronization primitives.

use axasync::{
    sleep,
    sync::{Barrier, Mutex, RwLock, Semaphore},
};
use axstd::println;
use axstd::time::Duration;
use axstd::vec;

/// Demonstrates the use of async mutex
pub async fn mutex_demo() {
    println!("\n=== Async Mutex Demo ===");

    // Create a mutex with an initial value
    let mutex = Mutex::new(0);

    // Acquire the mutex and modify the value
    {
        println!("Acquiring mutex...");
        let mut guard = mutex.lock().await;
        println!("Mutex acquired, current value: {}", *guard);
        *guard += 1;
        println!("Modified value to: {}", *guard);

        // Sleep while holding the mutex to demonstrate exclusive access
        println!("Sleeping for 100ms while holding the mutex...");
        sleep(Duration::from_millis(100)).await;

        println!("Releasing mutex...");
        // The mutex is automatically released when the guard is dropped
    }

    // Acquire the mutex again
    {
        println!("Acquiring mutex again...");
        let guard = mutex.lock().await;
        println!("Mutex acquired again, value is now: {}", *guard);
    }
}

/// Demonstrates the use of async RwLock
pub async fn rwlock_demo() {
    println!("\n=== Async RwLock Demo ===");

    // Create a RwLock with an initial value
    let rwlock = RwLock::new(vec![1, 2, 3]);

    // Acquire a read lock
    {
        println!("Acquiring read lock...");
        let read_guard = rwlock.read().await;
        println!("Read lock acquired, current value: {:?}", *read_guard);

        // Sleep while holding the read lock
        println!("Sleeping for 100ms while holding the read lock...");
        sleep(Duration::from_millis(100)).await;

        println!("Releasing read lock...");
        // The read lock is automatically released when the guard is dropped
    }

    // Acquire a write lock
    {
        println!("Acquiring write lock...");
        let mut write_guard = rwlock.write().await;
        println!("Write lock acquired, current value: {:?}", *write_guard);

        // Modify the value
        write_guard.push(4);
        println!("Modified value to: {:?}", *write_guard);

        // Sleep while holding the write lock
        println!("Sleeping for 100ms while holding the write lock...");
        sleep(Duration::from_millis(100)).await;

        println!("Releasing write lock...");
        // The write lock is automatically released when the guard is dropped
    }

    // Acquire a read lock again to verify the changes
    {
        println!("Acquiring read lock again...");
        let read_guard = rwlock.read().await;
        println!("Read lock acquired again, value is now: {:?}", *read_guard);
    }
}

/// Demonstrates the use of async semaphore
pub async fn semaphore_demo() {
    println!("\n=== Async Semaphore Demo ===");

    // Create a semaphore with 2 permits
    let semaphore = Semaphore::new(2);

    println!("Semaphore created with 2 permits");
    println!("Available permits: {}", semaphore.available_permits());

    // Acquire first permit
    println!("Acquiring first permit...");
    let permit1 = semaphore.acquire().await;
    println!("First permit acquired");
    println!("Available permits: {}", semaphore.available_permits());

    // Acquire second permit
    println!("Acquiring second permit...");
    let permit2 = semaphore.acquire().await;
    println!("Second permit acquired");
    println!("Available permits: {}", semaphore.available_permits());

    // Try to acquire a third permit (will fail)
    println!("Trying to acquire third permit immediately...");
    match semaphore.try_acquire() {
        Some(_) => println!("Third permit acquired (unexpected)"),
        None => println!("Failed to acquire third permit (expected)"),
    }

    // Release first permit
    println!("Releasing first permit...");
    drop(permit1);
    println!("Available permits: {}", semaphore.available_permits());

    // Release second permit
    println!("Releasing second permit...");
    drop(permit2);
    println!("Available permits: {}", semaphore.available_permits());
}

/// Demonstrates the use of async barrier
pub async fn barrier_demo() {
    println!("\n=== Async Barrier Demo ===");

    // Create a barrier (initially locked)
    let barrier = Barrier::new(true);

    println!("Barrier created (initially locked)");
    println!("Is barrier released? {}", barrier.is_released());

    // Try to acquire the barrier (will fail since it's locked)
    println!("Trying to acquire barrier immediately...");
    match barrier.try_acquire() {
        Some(_) => println!("Barrier acquired (unexpected)"),
        None => println!("Failed to acquire barrier (expected)"),
    }

    // Create a new barrier (initially unlocked)
    let barrier = Barrier::new(false);

    println!("\nNew barrier created (initially unlocked)");
    println!("Is barrier released? {}", barrier.is_released());

    // Acquire the barrier
    println!("Acquiring barrier...");
    let guard = barrier.acquire().await;
    println!("Barrier acquired");
    println!("Is barrier released? {}", barrier.is_released());

    // Explicitly release the barrier
    println!("Explicitly releasing barrier...");
    guard.release();
    println!("Is barrier released? {}", barrier.is_released());
}

/// Run all synchronization demos
pub async fn run_all_demos() {
    mutex_demo().await;
    rwlock_demo().await;
    semaphore_demo().await;
    barrier_demo().await;
}
