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

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(type_alias_impl_trait)]

#[macro_use]
extern crate axlog;

extern crate alloc;

use alloc::boxed::Box;
use core::future::Future;
use core::net::SocketAddr;
use core::pin::Pin;
use core::task::{Context, Poll};

pub mod sync;
pub mod time;
mod waker;

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
    events: core::cell::RefCell<
        heapless::BinaryHeap<TimerEventEntry<E>, heapless::binary_heap::Max, 32>,
    >,
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
            events: core::cell::RefCell::new(heapless::BinaryHeap::new()),
        }
    }

    pub fn set(&self, deadline: TimeValue, event: E) {
        let entry = TimerEventEntry { deadline, event };
        let mut events = self.events.borrow_mut();
        let _ = events.push(entry); // Ignore if the heap is full
    }

    pub fn expire_one(&self, now: TimeValue) -> Option<(TimeValue, E)> {
        let mut events = self.events.borrow_mut();
        if let Some(entry) = events.peek() {
            if entry.deadline <= now {
                if let Some(entry) = events.pop() {
                    return Some((entry.deadline, entry.event));
                }
            }
        }
        None
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

cfg_if::cfg_if! {
    if #[cfg(feature = "multitask")] {
        mod executor;
        pub use executor::*;
    }
}

/// Creates a new [`Waker`] that is a no-op.
pub fn dummy_waker() -> core::task::Waker {
    use core::task::{RawWaker, RawWakerVTable, Waker};

    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RawWaker::new(core::ptr::null(), &VTABLE),
        |_| {},
        |_| {},
        |_| {},
    );
    unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) }
}

/// Polls a future once, returning `Poll::Ready` if it completes.
pub fn poll_once<F>(fut: &mut F) -> core::task::Poll<F::Output>
where
    F: core::future::Future,
{
    use core::task::Context;
    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    unsafe { core::pin::Pin::new_unchecked(fut) }.poll(&mut cx)
}

/// Blocks on a future until it completes.
pub fn block_on<F>(mut fut: F) -> F::Output
where
    F: core::future::Future,
{
    use core::task::Poll;
    loop {
        match poll_once(&mut fut) {
            Poll::Ready(output) => return output,
            Poll::Pending => {
                // Yield the CPU if this future is not ready
                axtask::yield_now();
            }
        }
    }
}

/// Initialize the async runtime.
pub fn init() {
    info!("Async runtime initialized");
}

/// Shutdown the async runtime.
pub fn shutdown() {
    info!("Async runtime shut down");
}
