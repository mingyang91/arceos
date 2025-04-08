//! Asynchronous I/O traits and utilities for ArceOS.
//!
//! This module provides async abstractions for I/O operations, including file and network operations.
//! It is designed to work seamlessly with the ArceOS kernel's existing I/O infrastructure while
//! providing a non-blocking interface for async tasks.

use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};

#[cfg(feature = "alloc")]
use alloc::string::String;

use alloc::string::ToString;

mod buf;
mod error;
mod reactor;

pub use buf::{BufReader, BufWriter};
pub use error::{Error, ErrorKind, Result};
pub use reactor::{
    AsyncIoBackend, Completion, IoFuture, IoOperation, IoReactor, RequestId, global_reactor,
};

// Callback type for initialization and shutdown
type IoCallback = fn();

// Static initialization and shutdown functions
pub struct IoFunc {
    func: Option<IoCallback>,
}

impl IoFunc {
    pub const fn new() -> Self {
        Self { func: None }
    }

    pub fn set(&mut self, func: IoCallback) {
        self.func = Some(func);
    }

    pub fn get(&self) -> Option<IoCallback> {
        self.func
    }
}

// Module-level init and shutdown functions
pub static mut INIT_FUNC: IoFunc = IoFunc::new();
pub static mut SHUTDOWN_FUNC: IoFunc = IoFunc::new();
static IO_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "file")]
pub mod fs;

#[cfg(feature = "net")]
pub mod net;

/// Initialize the async I/O subsystem.
///
/// This function must be called before using any async I/O functionality.
pub fn init() {
    if IO_INITIALIZED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        reactor::init();
        unsafe {
            INIT_FUNC = IoFunc::new();
            INIT_FUNC.set(init);
        }
        unsafe {
            SHUTDOWN_FUNC = IoFunc::new();
            SHUTDOWN_FUNC.set(shutdown);
        }
    }
}

/// Shutdown the async I/O subsystem.
pub fn shutdown() {
    if IO_INITIALIZED
        .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        reactor::shutdown();
    }
}

/// Submit an I/O operation to the global reactor.
pub fn submit_operation(operation: IoOperation) -> Result<IoFuture> {
    let reactor = reactor::global_reactor();
    reactor.submit_operation(operation)
}

/// Represents an asynchronous read operation.
pub trait AsyncRead {
    /// Attempt to read data from the object into the specified buffer, returning how many bytes were read.
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8])
    -> Poll<Result<usize>>;

    /// Attempt to read exactly `buf.len()` bytes from the object into the buffer.
    fn poll_read_exact(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<()>> {
        // This is a utility function that keeps track of how many bytes have been read
        // and stops when the buffer is full or an error occurs
        let mut total_read = 0;

        while total_read < buf.len() {
            let remaining = &mut buf[total_read..];

            // Call poll_read with the correct pinning
            match self.as_mut().poll_read(cx, remaining) {
                Poll::Ready(Ok(0)) => return Poll::Ready(Err(error::Error::unexpected_eof())),
                Poll::Ready(Ok(n)) => total_read += n,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        Poll::Ready(Ok(()))
    }
}

/// Represents an asynchronous write operation.
pub trait AsyncWrite {
    /// Attempt to write data from the specified buffer into the object, returning how many bytes were written.
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>>;

    /// Attempt to write the entire contents of the buffer into the object.
    fn poll_write_all(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<()>> {
        // This is a utility function that keeps track of how many bytes have been written
        // and stops when the entire buffer is written or an error occurs
        let mut total_written = 0;

        while total_written < buf.len() {
            let remaining = &buf[total_written..];

            // Call poll_write with the correct pinning
            match self.as_mut().poll_write(cx, remaining) {
                Poll::Ready(Ok(0)) => {
                    return Poll::Ready(Err(io_error(ErrorKind::WriteZero, "write zero bytes")));
                }
                Poll::Ready(Ok(n)) => total_written += n,
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        Poll::Ready(Ok(()))
    }

    /// Attempt to flush the object, ensuring all intermediately buffered contents reach their destination.
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>>;

    /// Attempt to close the object.
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>>;
}

/// Extension methods for `AsyncRead` types.
pub trait AsyncReadExt: AsyncRead {
    /// Read all bytes until EOF in this source, placing them into `buf`.
    fn read_to_end<'a>(
        &'a mut self,
        buf: &'a mut Vec<u8>,
    ) -> impl Future<Output = Result<usize>> + 'a
    where
        Self: Unpin,
    {
        ReadToEnd { reader: self, buf }
    }

    /// Read exactly `buf.len()` bytes from the source into the buffer.
    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        ReadExact { reader: self, buf }
    }

    /// Creates an adapter which will chain this stream with another.
    fn chain<R>(self, next: R) -> Chain<Self, R>
    where
        Self: Sized,
        R: AsyncRead,
    {
        Chain {
            first: self,
            second: next,
            done_first: false,
        }
    }

    /// Creates a buffered reader.
    fn buffered(self, capacity: usize) -> BufReader<Self>
    where
        Self: Sized,
    {
        BufReader::with_capacity(capacity, self)
    }
}

impl<R: AsyncRead + ?Sized> AsyncReadExt for R {}

/// Extension methods for `AsyncWrite` types.
pub trait AsyncWriteExt: AsyncWrite {
    /// Write the entire contents of the buffer into the object.
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        WriteAll { writer: self, buf }
    }

    /// Flush the object, ensuring all intermediately buffered contents reach their destination.
    fn flush<'a>(&'a mut self) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        Flush { writer: self }
    }

    /// Close the object.
    fn close<'a>(&'a mut self) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        Close { writer: self }
    }

    /// Creates a buffered writer.
    fn buffered(self, capacity: usize) -> BufWriter<Self>
    where
        Self: Sized,
    {
        BufWriter::with_capacity(capacity, self)
    }
}

impl<W: AsyncWrite + ?Sized> AsyncWriteExt for W {}

// Future structs for AsyncReadExt/AsyncWriteExt methods
struct ReadToEnd<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut Vec<u8>,
}

impl<R: AsyncRead + ?Sized + Unpin> Future for ReadToEnd<'_, R> {
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let mut buf = [0; 1024];
        let mut total_read = 0;

        loop {
            let reader = Pin::new(&mut *this.reader);
            match reader.poll_read(cx, &mut buf) {
                Poll::Ready(Ok(0)) => return Poll::Ready(Ok(total_read)),
                Poll::Ready(Ok(n)) => {
                    total_read += n;
                    this.buf.extend_from_slice(&buf[..n]);
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

struct ReadExact<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut [u8],
}

impl<R: AsyncRead + ?Sized + Unpin> Future for ReadExact<'_, R> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let reader = Pin::new(&mut *this.reader);
        reader.poll_read_exact(cx, this.buf)
    }
}

struct WriteAll<'a, W: ?Sized> {
    writer: &'a mut W,
    buf: &'a [u8],
}

impl<W: AsyncWrite + ?Sized + Unpin> Future for WriteAll<'_, W> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let writer = Pin::new(&mut *this.writer);
        writer.poll_write_all(cx, this.buf)
    }
}

struct Flush<'a, W: ?Sized> {
    writer: &'a mut W,
}

impl<W: AsyncWrite + ?Sized + Unpin> Future for Flush<'_, W> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let writer = Pin::new(&mut *this.writer);
        writer.poll_flush(cx)
    }
}

struct Close<'a, W: ?Sized> {
    writer: &'a mut W,
}

impl<W: AsyncWrite + ?Sized + Unpin> Future for Close<'_, W> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let writer = Pin::new(&mut *this.writer);
        writer.poll_close(cx)
    }
}

/// An adapter to chain two readers together.
pub struct Chain<T, U> {
    first: T,
    second: U,
    done_first: bool,
}

impl<T, U> Unpin for Chain<T, U>
where
    T: Unpin,
    U: Unpin,
{
}

impl<T, U> AsyncRead for Chain<T, U>
where
    T: AsyncRead + Unpin,
    U: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        if !self.done_first {
            match Pin::new(&mut self.first).poll_read(cx, buf) {
                Poll::Ready(Ok(0)) => {
                    self.done_first = true;
                }
                Poll::Ready(Ok(n)) => return Poll::Ready(Ok(n)),
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        Pin::new(&mut self.second).poll_read(cx, buf)
    }
}

/// Creates an I/O error with a specific kind and message.
pub fn io_error(kind: ErrorKind, message: &'static str) -> Error {
    Error::new(kind, message.to_string())
}
