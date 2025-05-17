//! Async runtime for [ArceOS](https://github.com/arceos-org/arceos).
//!
//! This module provides a lightweight async runtime that resembles the design of
//! [embassy-rs](https://github.com/embassy-rs/embassy), but tailored for ArceOS.
//!
//! # Cargo Features
//!
//! - `multitask`: Enable multi-task support.
//! - `irq`: Enable interrupt handling support.
//! - `timer`: Enable async timer functionality (requires `irq`).
//! - `file`: Enable async filesystem functionality.
//! - `net`: Enable async networking functionality.
//! - `mmio`: Enable async MMIO functionality (requires `irq`).

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(type_alias_impl_trait)]

#[macro_use]
extern crate axlog;

extern crate alloc;

pub mod executor;
pub mod sync;
pub mod time;
mod waker;
use alloc::collections::BinaryHeap;
use core::pin::Pin;
use core::task::{Context, Poll};

#[cfg(feature = "mmio")]
pub mod mmio;

pub use executor::{
    BoxFuture,
    Executor,
    JoinHandle,
    // Global executor functions
    block_on,
    dummy_waker,
    executor,
    init as executor_init,
    poll_once,
    run as executor_run,
    run_local,
    spawn,
    spawn_local,
};
pub use futures_util;
pub use time::{TimeoutExt, sleep};
pub use waker::*;

// Timer event definition for our TimerList implementation
#[cfg(feature = "timer")]
use axhal::time::TimeValue;

#[cfg(feature = "timer")]
pub trait TimerEvent {
    fn callback(self, now: TimeValue);
}

#[cfg(feature = "timer")]
pub struct TimerList<E: TimerEvent> {
    events: core::cell::RefCell<BinaryHeap<TimerEventEntry<E>>>,
}

#[cfg(feature = "timer")]
struct TimerEventEntry<E: TimerEvent> {
    deadline: TimeValue,
    event: E,
}

#[cfg(feature = "timer")]
impl<E: TimerEvent> TimerList<E> {
    pub fn new() -> Self {
        Self {
            events: core::cell::RefCell::new(BinaryHeap::new()),
        }
    }

    pub fn set(&self, deadline: TimeValue, event: E) {
        {
            let entry = TimerEventEntry { deadline, event };
            let mut events = self.events.borrow_mut();
            let _ = events.push(entry); // Ignore if the heap is full
        }
        // self.set_timer();
    }

    pub fn expire_one(&self, now: TimeValue) -> Option<(TimeValue, E)> {
        let entry = {
            let mut events = self.events.borrow_mut();
            let Some(entry) = events.peek() else {
                return None;
            };
            if entry.deadline > now {
                return None;
            }
            let Some(entry) = events.pop() else {
                return None;
            };
            entry
        };
        // self.set_timer();
        Some((entry.deadline, entry.event))
    }

    fn set_timer(&self) {
        if let Some(entry) = self.events.borrow().peek() {
            debug!("Setting timer for {:?}", entry.deadline);
            axhal::time::set_oneshot_timer(entry.deadline.as_nanos() as u64);
        }
    }
}

#[cfg(feature = "timer")]
impl<E: TimerEvent> core::cmp::Ord for TimerEventEntry<E> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deadline.cmp(&other.deadline)
    }
}

#[cfg(feature = "timer")]
impl<E: TimerEvent> core::cmp::PartialOrd for TimerEventEntry<E> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "timer")]
impl<E: TimerEvent> core::cmp::PartialEq for TimerEventEntry<E> {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline
    }
}

#[cfg(feature = "timer")]
impl<E: TimerEvent> core::cmp::Eq for TimerEventEntry<E> {}

/// Initialize the async runtime.
pub fn init() {
    executor_init();
    info!("Async runtime initialized");
}

/// Shutdown the async runtime.
pub fn shutdown() {
    info!("Async runtime shut down");
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::sync::Arc;
    use core::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_global_spawn() {
        // Initialize runtime
        init();

        // Create a shared completion flag
        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        // Spawn a task using the global spawn function
        let _handle = spawn(async move {
            completed_clone.store(true, Ordering::SeqCst);
        });

        // Run the executor
        executor_run();

        // Verify the task completed
        assert!(completed.load(Ordering::SeqCst));
    }
}
