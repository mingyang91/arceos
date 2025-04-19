#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use axstd::{io, println, thread, time::Duration};
use core::sync::atomic::{AtomicBool, Ordering};

#[no_mangle]
fn main() -> io::Result<()> {
    println!("MMIO Async Demo");
    println!("MMIO Async Demo completed!");
    Ok(())
}
