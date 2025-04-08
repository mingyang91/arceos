//! Asynchronous networking I/O traits and implementations for ArceOS.
//!
//! This module provides async abstractions for network operations such as TCP and UDP
//! sockets, building on top of the blocking network interface provided by `axnet`.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use axerrno::{AxError, AxResult};
use axnet::TcpSocket as SyncTcpSocket;
use axnet::UdpSocket as SyncUdpSocket;
use axsync::Mutex;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};

use super::{AsyncRead, AsyncWrite, Error, IoFuture, IoOperation, Result, submit_operation};

/// An asynchronous version of the TCP socket.
pub struct TcpSocket {
    inner: Arc<Mutex<SyncTcpSocket>>,
}

impl TcpSocket {
    /// Creates a new TCP socket.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SyncTcpSocket::new())),
        }
    }

    /// Returns the local address and port.
    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        self.inner.lock().local_addr()
    }

    /// Returns the remote address and port.
    pub fn peer_addr(&self) -> AxResult<SocketAddr> {
        self.inner.lock().peer_addr()
    }

    /// Connects to the given address.
    pub fn connect(&self, addr: SocketAddr) -> IoFuture<Result<()>> {
        let socket = self.inner.clone();
        let operation = IoOperation::Connect { socket, addr };
        match submit_operation(operation) {
            Ok(future) => future,
            Err(e) => IoFuture::from_error(e),
        }
    }

    /// Binds the socket to the given address.
    pub fn bind(&self, addr: SocketAddr) -> AxResult {
        self.inner.lock().bind(addr)
    }

    /// Starts listening for incoming connections.
    pub fn listen(&self) -> AxResult {
        self.inner.lock().listen()
    }

    /// Accepts a new incoming connection.
    pub fn accept(&self) -> IoFuture<Result<TcpSocket>> {
        let socket = self.inner.clone();
        let operation = IoOperation::Accept { socket };
        match submit_operation(operation) {
            Ok(future) => future.map(|res| {
                res.map(|inner| TcpSocket {
                    inner: Arc::new(Mutex::new(inner)),
                })
            }),
            Err(e) => IoFuture::from_error(e),
        }
    }

    /// Shuts down the socket.
    pub fn shutdown(&self) -> AxResult {
        self.inner.lock().shutdown()
    }
}

impl AsyncRead for TcpSocket {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        let socket = self.inner.clone();
        let operation = IoOperation::Read {
            socket,
            buf: buf.as_ptr() as usize,
            len: buf.len(),
        };

        match submit_operation(operation) {
            Ok(mut future) => {
                // Poll the future directly
                Pin::new(&mut future).poll(cx)
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl AsyncWrite for TcpSocket {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let socket = self.inner.clone();
        let operation = IoOperation::Write {
            socket,
            buf: buf.as_ptr() as usize,
            len: buf.len(),
        };

        match submit_operation(operation) {
            Ok(mut future) => {
                // Poll the future directly
                Pin::new(&mut future).poll(cx)
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        // TCP sockets don't need explicit flushing
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.inner.lock().shutdown() {
            Ok(()) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(Err(Error::from(e))),
        }
    }
}

/// An asynchronous version of the UDP socket.
pub struct UdpSocket {
    inner: Arc<Mutex<SyncUdpSocket>>,
}

impl UdpSocket {
    /// Creates a new UDP socket.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SyncUdpSocket::new())),
        }
    }

    /// Returns the local address and port.
    pub fn local_addr(&self) -> AxResult<SocketAddr> {
        self.inner.lock().local_addr()
    }

    /// Binds the socket to the given address.
    pub fn bind(&self, addr: SocketAddr) -> AxResult {
        self.inner.lock().bind(addr)
    }

    /// Connects the socket to a remote address.
    pub fn connect(&self, addr: SocketAddr) -> AxResult {
        self.inner.lock().connect(addr)
    }

    /// Sends data to the socket's connected address.
    pub fn send(&self, buf: &[u8]) -> IoFuture<Result<usize>> {
        let socket = self.inner.clone();
        let operation = IoOperation::Send {
            socket,
            buf: buf.as_ptr() as usize,
            len: buf.len(),
        };

        match submit_operation(operation) {
            Ok(future) => future,
            Err(e) => IoFuture::from_error(e),
        }
    }

    /// Sends data to the specified address.
    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> IoFuture<Result<usize>> {
        let socket = self.inner.clone();
        let operation = IoOperation::SendTo {
            socket,
            buf: buf.as_ptr() as usize,
            len: buf.len(),
            addr,
        };

        match submit_operation(operation) {
            Ok(future) => future,
            Err(e) => IoFuture::from_error(e),
        }
    }

    /// Receives data from the socket's connected address.
    pub fn recv(&self, buf: &mut [u8]) -> IoFuture<Result<usize>> {
        let socket = self.inner.clone();
        let operation = IoOperation::Recv {
            socket,
            buf: buf.as_ptr() as usize,
            len: buf.len(),
        };

        match submit_operation(operation) {
            Ok(future) => future,
            Err(e) => IoFuture::from_error(e),
        }
    }

    /// Receives data from any address.
    pub fn recv_from(&self, buf: &mut [u8]) -> IoFuture<Result<(usize, SocketAddr)>> {
        let socket = self.inner.clone();
        let operation = IoOperation::RecvFrom {
            socket,
            buf: buf.as_ptr() as usize,
            len: buf.len(),
        };

        match submit_operation(operation) {
            Ok(future) => future,
            Err(e) => IoFuture::from_error(e),
        }
    }
}

/// Extension traits for TCP sockets
pub trait TcpSocketExt {
    /// Creates a new connection to the specified address.
    fn connect_to(addr: SocketAddr) -> IoFuture<Result<TcpSocket>>;
}

impl TcpSocketExt for TcpSocket {
    fn connect_to(addr: SocketAddr) -> IoFuture<Result<TcpSocket>> {
        let socket = TcpSocket::new();
        let connect_future = socket.connect(addr);

        async move {
            connect_future.await?;
            Ok(socket)
        }
        .boxed_local()
    }
}

/// Trait extension for Future to support boxing
trait FutureExt: Future + Sized {
    fn boxed_local<'a>(self) -> LocalBoxFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(self)
    }
}

impl<F: Future + Sized> FutureExt for F {}

/// A type alias for locally boxed futures
type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
