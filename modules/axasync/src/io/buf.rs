//! Buffered I/O types.

use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::{AsyncRead, AsyncWrite, Result};

/// A buffered reader that implements `AsyncRead`.
pub struct BufReader<R> {
    inner: R,
    buf: Vec<u8>,
    pos: usize,
    cap: usize,
}

impl<R> BufReader<R> {
    /// Creates a new `BufReader` with a default capacity.
    pub fn new(inner: R) -> Self {
        Self::with_capacity(8 * 1024, inner)
    }

    /// Creates a new `BufReader` with the specified capacity.
    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        Self {
            inner,
            buf: Vec::with_capacity(capacity),
            pos: 0,
            cap: 0,
        }
    }

    /// Gets a reference to the underlying reader.
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Consumes this `BufReader`, returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Returns a reference to the internally buffered data.
    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.pos..self.cap]
    }

    /// Invalidates all data in the internal buffer.
    pub fn discard_buffer(&mut self) {
        self.pos = 0;
        self.cap = 0;
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for BufReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        // If we don't have any buffered data and we're doing a non-zeroing read
        // (for example, if we just want to fill out the read_vec), then skip
        // the buffering entirely.
        if self.pos == self.cap && !buf.is_empty() {
            return Pin::new(&mut self.inner).poll_read(cx, buf);
        }
        let mut rem = self.buffer();
        if rem.is_empty() {
            // Ensure the buffer has capacity
            if self.buf.len() < self.buf.capacity() {
                self.buf.resize(self.buf.capacity(), 0);
            }

            // We need to read some data
            let buf_mut = &mut self.buf[..];
            let read_result = Pin::new(&mut self.inner).poll_read(cx, buf_mut);
            match read_result {
                Poll::Ready(Ok(n)) => {
                    self.pos = 0;
                    self.cap = n;
                    rem = &self.buf[..self.cap];
                    if rem.is_empty() {
                        return Poll::Ready(Ok(0));
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        // We have some data in the buffer, copy it
        let amt = core::cmp::min(buf.len(), rem.len());
        buf[..amt].copy_from_slice(&rem[..amt]);
        self.pos += amt;
        Poll::Ready(Ok(amt))
    }
}

impl<R: AsyncRead + AsyncWrite + Unpin> AsyncWrite for BufReader<R> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

/// A buffered writer that implements `AsyncWrite`.
pub struct BufWriter<W> {
    inner: W,
    buf: Vec<u8>,
}

impl<W> BufWriter<W> {
    /// Creates a new `BufWriter` with a default capacity.
    pub fn new(inner: W) -> Self {
        Self::with_capacity(8 * 1024, inner)
    }

    /// Creates a new `BufWriter` with the specified capacity.
    pub fn with_capacity(capacity: usize, inner: W) -> Self {
        Self {
            inner,
            buf: Vec::with_capacity(capacity),
        }
    }

    /// Gets a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    /// Gets a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Consumes this `BufWriter`, returning the underlying writer.
    ///
    /// Note that any leftover data in the internal buffer is lost.
    pub fn into_inner(self) -> W {
        self.inner
    }

    /// Returns a reference to the internally buffered data.
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }
}

impl<W: AsyncWrite + Unpin> BufWriter<W> {
    /// Flush the buffer if it's full, returning any error that occurs.
    fn flush_buf(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        /// Helper struct to ensure the buffer is flushed
        struct BufGuard<'a, W: AsyncWrite + Unpin> {
            writer: &'a mut BufWriter<W>,
            written: usize,
        }

        impl<W: AsyncWrite + Unpin> Drop for BufGuard<'_, W> {
            fn drop(&mut self) {
                // Update the buffer position if some data was written
                if self.written > 0 {
                    let buf = &mut self.writer.buf;
                    let remaining = buf.len() - self.written;
                    if remaining > 0 {
                        buf.copy_within(self.written.., 0);
                    }
                    buf.truncate(remaining);
                }
            }
        }

        // If the buffer is empty, there's nothing to flush
        if self.buf.is_empty() {
            return Poll::Ready(Ok(()));
        }

        let mut guard = BufGuard {
            writer: self,
            written: 0,
        };
        let buf = &guard.writer.buf;

        loop {
            match Pin::new(&mut guard.writer.inner).poll_write(cx, &buf[guard.written..]) {
                Poll::Ready(Ok(0)) => {
                    // If we couldn't write anything, but the buffer is not empty,
                    // the writer has no more capacity
                    if guard.written == 0 {
                        return Poll::Ready(Err(super::io_error(
                            super::ErrorKind::WriteZero,
                            "failed to write to buffer",
                        )));
                    }
                    // Some data was written, but we can't write more now
                    return Poll::Ready(Ok(()));
                }
                Poll::Ready(Ok(n)) => {
                    guard.written += n;
                    // If we've written the whole buffer, we're done
                    if guard.written == buf.len() {
                        guard.writer.buf.clear();
                        return Poll::Ready(Ok(()));
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => {
                    // If we've written some data, that's progress
                    if guard.written > 0 {
                        return Poll::Ready(Ok(()));
                    }
                    return Poll::Pending;
                }
            }
        }
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for BufWriter<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        // If the buffer is already full, flush it first
        if self.buf.len() >= self.buf.capacity() {
            match self.flush_buf(cx) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        // If the input is large enough, bypass the buffer
        if buf.len() >= self.buf.capacity() {
            // Flush any existing data first
            if !self.buf.is_empty() {
                match self.flush_buf(cx) {
                    Poll::Ready(Ok(())) => {}
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Pending => return Poll::Pending,
                }
            }
            // Write directly to the underlying writer
            return Pin::new(&mut self.inner).poll_write(cx, buf);
        }

        // Otherwise, buffer the data
        let available = self.buf.capacity() - self.buf.len();
        let amt = core::cmp::min(available, buf.len());
        self.buf.extend_from_slice(&buf[..amt]);
        Poll::Ready(Ok(amt))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        // First flush our buffer
        match self.flush_buf(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        }
        // Then flush the underlying writer
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        // First flush our buffer
        match self.flush_buf(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        }
        // Then close the underlying writer
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

impl<W: AsyncRead + Unpin> AsyncRead for BufWriter<W>
where
    W: AsyncRead,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}
