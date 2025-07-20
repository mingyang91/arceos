use super::addr::from_core_sockaddr;
use crate::net_impl::{ETH0, LISTEN_TABLE, SOCKET_SET, SocketSetWrapper};
use crate::smoltcp_impl::tcp::{STATE_CLOSED, STATE_CONNECTING};
use axio::PollState;
use core::future::Future;
use core::net::SocketAddr;
use core::pin::Pin;
use core::task::{Context, Poll};
use smoltcp::socket::tcp::{ConnectError, Socket};

use axerrno::{AxError, AxResult, ax_err, ax_err_type};

use super::TcpSocket;

pub struct RecvFuture<'a> {
    socket: &'a TcpSocket,
    buf: &'a mut [u8],
    init: bool,
}

impl<'a> RecvFuture<'a> {
    pub fn new(socket: &'a TcpSocket, buf: &'a mut [u8]) -> Self {
        Self {
            socket,
            buf,
            init: false,
        }
    }
}

impl<'a> Future for RecvFuture<'a> {
    type Output = AxResult<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        trace!("recv poll");
        let this = self.get_mut();
        if !this.init {
            this.init = true;
            if this.socket.is_connecting() {
                return Poll::Ready(Err(AxError::WouldBlock));
            } else if !this.socket.is_connected() {
                return Poll::Ready(ax_err!(NotConnected, "socket recv() failed"));
            }
        }

        let handle = this.socket.handle();
        SOCKET_SET.with_socket_mut::<Socket, _, _>(handle, |socket| {
            if !socket.is_active() {
                return Poll::Ready(ax_err!(ConnectionRefused, "socket recv() failed"));
            } else if !socket.may_recv() {
                return Poll::Ready(Ok(0));
            } else if socket.recv_queue() > 0 {
                return Poll::Ready(
                    socket
                        .recv_slice(this.buf)
                        .map_err(|_| ax_err_type!(BadState, "socket recv() failed")),
                );
            } else {
                socket.register_recv_waker(cx.waker());
                return Poll::Pending;
            }
        })
    }
}

pub struct SendFuture<'a> {
    socket: &'a TcpSocket,
    buf: &'a [u8],
    init: bool,
}

impl<'a> SendFuture<'a> {
    pub fn new(socket: &'a TcpSocket, buf: &'a [u8]) -> Self {
        Self {
            socket,
            buf,
            init: false,
        }
    }
}

impl<'a> Future for SendFuture<'a> {
    type Output = AxResult<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        trace!("send poll");
        let this = self.get_mut();
        if !this.init {
            this.init = true;
            if this.socket.is_connecting() {
                return Poll::Ready(Err(AxError::WouldBlock));
            } else if !this.socket.is_connected() {
                return Poll::Ready(ax_err!(NotConnected, "socket send() failed"));
            }
        }

        let handle = this.socket.handle();
        SOCKET_SET.with_socket_mut::<Socket, _, _>(handle, |socket| {
            if !socket.is_active() || !socket.may_send() {
                return Poll::Ready(ax_err!(ConnectionReset, "socket send() failed"));
            } else if socket.can_send() {
                return Poll::Ready(
                    socket
                        .send_slice(this.buf)
                        .map_err(|_| ax_err_type!(BadState, "socket send() failed")),
                );
            } else {
                socket.register_send_waker(cx.waker());
                return Poll::Pending;
            }
        })
    }
}

pub struct AcceptFuture<'a> {
    socket: &'a TcpSocket,
    init: bool,
}

impl<'a> AcceptFuture<'a> {
    pub fn new(socket: &'a TcpSocket) -> Self {
        Self {
            socket,
            init: false,
        }
    }
}

impl<'a> Future for AcceptFuture<'a> {
    type Output = AxResult<TcpSocket>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if !this.init {
            this.init = true;
            if !this.socket.is_listening() {
                return Poll::Ready(ax_err!(InvalidInput, "socket accept() failed"));
            }
        }

        // SOCKET_SET.poll_interfaces();
        let local_port = this.socket.local_addr().unwrap().port();
        let (handle, (local_addr, peer_addr)) = match LISTEN_TABLE.accept(local_port) {
            Ok(res) => res,
            Err(e) if e == AxError::WouldBlock => {
                return Poll::Pending;
            }
            Err(e) => return Poll::Ready(ax_err!(e)),
        };

        trace!("TCP socket accepted a new connection {}", peer_addr);
        Poll::Ready(Ok(TcpSocket::new_connected(handle, local_addr, peer_addr)))
    }
}

pub struct ConnectFuture<'a> {
    socket: &'a TcpSocket,
    remote_addr: SocketAddr,
    init: bool,
}

impl<'a> ConnectFuture<'a> {
    pub fn new(socket: &'a TcpSocket, remote_addr: SocketAddr) -> Self {
        Self {
            socket,
            remote_addr,
            init: false,
        }
    }
}

impl<'a> Future for ConnectFuture<'a> {
    type Output = AxResult<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        trace!("connect poll");
        let this = self.get_mut();
        if !this.init {
            this.init = true;
            if let Err(e) = this
                .socket
                .update_state(STATE_CLOSED, STATE_CONNECTING, || {
                    // SAFETY: no other threads can read or write these fields.
                    let handle = unsafe { this.socket.handle.get().read() }
                        .unwrap_or_else(|| SOCKET_SET.add(SocketSetWrapper::new_tcp_socket()));

                    // TODO: check remote addr unreachable
                    let remote_endpoint = from_core_sockaddr(this.remote_addr);
                    let bound_endpoint = this.socket.bound_endpoint()?;
                    let iface = &ETH0.iface;
                    let (local_endpoint, remote_endpoint) = SOCKET_SET
                        .with_socket_mut::<Socket, _, _>(handle, |socket| {
                            socket
                                .connect(iface.lock().context(), remote_endpoint, bound_endpoint)
                                .or_else(|e| match e {
                                    ConnectError::InvalidState => {
                                        ax_err!(BadState, "socket connect() failed")
                                    }
                                    ConnectError::Unaddressable => {
                                        ax_err!(ConnectionRefused, "socket connect() failed")
                                    }
                                })?;
                            Ok((
                                socket.local_endpoint().unwrap(),
                                socket.remote_endpoint().unwrap(),
                            ))
                        })?;
                    unsafe {
                        // SAFETY: no other threads can read or write these fields as we
                        // have changed the state to `BUSY`.
                        this.socket.local_addr.get().write(local_endpoint);
                        this.socket.peer_addr.get().write(remote_endpoint);
                        this.socket.handle.get().write(Some(handle));
                    }
                    Ok(())
                })
                .unwrap_or_else(|_| {
                    ax_err!(AlreadyExists, "socket connect() failed: already connected")
                })
            {
                // EISCONN
                return Poll::Ready(ax_err!(e));
            }
        }

        let PollState { writable, .. } = this.socket.poll_connect()?;
        if !writable {
            let handle = unsafe {
                this.socket
                    .handle
                    .get()
                    .read()
                    .expect("handle should be initialized")
            };
            SOCKET_SET.with_socket_mut::<Socket, _, _>(handle, |socket| {
                socket.register_recv_waker(cx.waker());
            });
            return Poll::Pending;
        } else if this.socket.is_connected() {
            return Poll::Ready(Ok(()));
        } else {
            return Poll::Ready(ax_err!(ConnectionRefused, "socket connect() failed"));
        }
    }
}
