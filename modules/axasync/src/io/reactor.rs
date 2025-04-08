//! I/O reactor implementation for async I/O operations.
//!
//! This module provides the core functionality for registering and handling
//! asynchronous I/O operations.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};

use axsync::Mutex;
use core::net::SocketAddr;

use super::{Error, Result};

/// A unique identifier for an I/O request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestId(u64);

impl RequestId {
    fn next() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// An I/O operation that can be submitted to the reactor.
pub enum IoOperation {
    /// Read operation for a file or socket.
    Read {
        /// The socket/file to read from (stored as Arc<Mutex<T>>).
        socket: Arc<Mutex<dyn core::any::Any + Send + Sync>>,
        /// The buffer pointer to read into.
        buf: usize,
        /// The length of the buffer.
        len: usize,
    },
    /// Write operation for a file or socket.
    Write {
        /// The socket/file to write to (stored as Arc<Mutex<T>>).
        socket: Arc<Mutex<dyn core::any::Any + Send + Sync>>,
        /// The buffer pointer to write from.
        buf: usize,
        /// The length of the buffer.
        len: usize,
    },
    /// Connect to a remote address.
    Connect {
        /// The socket to connect with (stored as Arc<Mutex<T>>).
        socket: Arc<Mutex<dyn core::any::Any + Send + Sync>>,
        /// The address to connect to.
        addr: SocketAddr,
    },
    /// Accept a new connection.
    Accept {
        /// The listener socket (stored as Arc<Mutex<T>>).
        socket: Arc<Mutex<dyn core::any::Any + Send + Sync>>,
    },
    /// Send data on a socket.
    Send {
        /// The socket to send with (stored as Arc<Mutex<T>>).
        socket: Arc<Mutex<dyn core::any::Any + Send + Sync>>,
        /// The buffer pointer to send from.
        buf: usize,
        /// The length of the buffer.
        len: usize,
    },
    /// Send data on a socket to a specific address.
    SendTo {
        /// The socket to send with (stored as Arc<Mutex<T>>).
        socket: Arc<Mutex<dyn core::any::Any + Send + Sync>>,
        /// The buffer pointer to send from.
        buf: usize,
        /// The length of the buffer.
        len: usize,
        /// The destination address.
        addr: SocketAddr,
    },
    /// Receive data from a socket.
    Recv {
        /// The socket to receive from (stored as Arc<Mutex<T>>).
        socket: Arc<Mutex<dyn core::any::Any + Send + Sync>>,
        /// The buffer pointer to receive into.
        buf: usize,
        /// The length of the buffer.
        len: usize,
    },
    /// Receive data from a socket, along with the source address.
    RecvFrom {
        /// The socket to receive from (stored as Arc<Mutex<T>>).
        socket: Arc<Mutex<dyn core::any::Any + Send + Sync>>,
        /// The buffer pointer to receive into.
        buf: usize,
        /// The length of the buffer.
        len: usize,
    },
}

/// The result of an I/O operation.
pub enum Completion {
    /// A successful read operation.
    Read(usize),
    /// A successful write operation.
    Write(usize),
    /// A successful connect operation.
    Connect,
    /// A successful accept operation, returning the accepted socket.
    Accept(Box<dyn core::any::Any + Send + Sync>),
    /// A successful send operation.
    Send(usize),
    /// A successful send_to operation.
    SendTo(usize),
    /// A successful recv operation.
    Recv(usize),
    /// A successful recv_from operation, returning the bytes read and the source address.
    RecvFrom(usize, SocketAddr),
    /// An error occurred during the operation.
    Error(Error),
}

/// A future that represents a pending I/O operation.
pub struct IoFuture {
    id: RequestId,
    state: Arc<UnsafeCell<IoFutureState>>,
}

struct IoFutureState {
    result: Option<Completion>,
    waker: Option<Waker>,
}

impl IoFuture {
    fn new(id: RequestId) -> Self {
        Self {
            id,
            state: Arc::new(UnsafeCell::new(IoFutureState {
                result: None,
                waker: None,
            })),
        }
    }

    /// Creates a new future with an immediate error result.
    pub fn from_error(error: Error) -> Self {
        let mut future = Self::new(RequestId::next());
        unsafe {
            let state = &mut *future.state.get();
            state.result = Some(Completion::Error(error));
        }
        future
    }

    /// Completes this future with the given result.
    fn complete(&self, result: Completion) {
        unsafe {
            let state = &mut *self.state.get();
            state.result = Some(result);
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        }
    }

    /// Maps the result of this future using the given function.
    pub fn map<F, T>(self, f: F) -> IoFutureMap<F, T>
    where
        F: FnOnce(Result<Completion>) -> T,
    {
        IoFutureMap {
            future: self,
            f: Some(f),
            _marker: core::marker::PhantomData,
        }
    }
}

impl Future for IoFuture {
    type Output = Result<Completion>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let state = &mut *self.state.get();
            if let Some(result) = state.result.take() {
                match result {
                    Completion::Error(e) => Poll::Ready(Err(e)),
                    completion => Poll::Ready(Ok(completion)),
                }
            } else {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

/// A future that maps the result of an `IoFuture`.
pub struct IoFutureMap<F, T> {
    future: IoFuture,
    f: Option<F>,
    _marker: core::marker::PhantomData<fn() -> T>,
}

impl<F, T> Future for IoFutureMap<F, T>
where
    F: FnOnce(Result<Completion>) -> T,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safety: we're not moving any fields around
        let this = unsafe { self.as_mut().get_unchecked_mut() };
        let mut future = unsafe { Pin::new_unchecked(&mut this.future) };
        match future.as_mut().poll(cx) {
            Poll::Ready(result) => {
                let f = this.f.take().expect("IoFutureMap polled after completion");
                Poll::Ready(f(result))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// The backend for handling I/O operations.
pub trait AsyncIoBackend: Send + Sync {
    /// Submits an I/O operation for processing.
    fn submit(&self, id: RequestId, operation: IoOperation);

    /// Processes any completed I/O operations.
    fn poll(&self) -> Vec<(RequestId, Completion)>;
}

/// The I/O reactor for handling async I/O operations.
pub struct IoReactor {
    backend: Box<dyn AsyncIoBackend>,
    operations: Mutex<VecDeque<(RequestId, Weak<UnsafeCell<IoFutureState>>)>>,
}

impl IoReactor {
    /// Creates a new I/O reactor with the given backend.
    pub fn new(backend: impl AsyncIoBackend + 'static) -> Self {
        Self {
            backend: Box::new(backend),
            operations: Mutex::new(VecDeque::new()),
        }
    }

    /// Submits an I/O operation to the reactor and returns a future for the result.
    pub fn submit_operation(&self, operation: IoOperation) -> Result<IoFuture> {
        let id = RequestId::next();
        let future = IoFuture::new(id);

        // Store the future's state so we can complete it later
        self.operations
            .lock()
            .push_back((id, Arc::downgrade(&future.state)));

        // Submit the operation to the backend
        self.backend.submit(id, operation);

        Ok(future)
    }

    /// Polls the reactor for completed operations.
    pub fn poll(&self) {
        // Get completed operations from the backend
        let completions = self.backend.poll();
        if completions.is_empty() {
            return;
        }

        // Complete the corresponding futures
        let mut operations = self.operations.lock();
        for (id, completion) in completions {
            let mut i = 0;
            while i < operations.len() {
                if operations[i].0 == id {
                    let (_, state_weak) = operations.remove(i).unwrap();
                    if let Some(state) = state_weak.upgrade() {
                        let future = IoFuture { id, state };
                        future.complete(completion);
                        break;
                    }
                } else {
                    i += 1;
                }
            }
        }

        // Cleanup any operations with dropped futures
        operations.retain(|(_, state_weak)| state_weak.upgrade().is_some());
    }
}

// Global I/O reactor instance
static mut GLOBAL_REACTOR: Option<IoReactor> = None;

/// Initialize the global I/O reactor with a default backend.
pub fn init() {
    // Here we would typically initialize with a real backend
    // For now, we use a dummy implementation
    let backend = DummyBackend::new();
    unsafe {
        GLOBAL_REACTOR = Some(IoReactor::new(backend));
    }
}

/// Shuts down the global I/O reactor.
pub fn shutdown() {
    unsafe {
        GLOBAL_REACTOR = None;
    }
}

/// Returns a reference to the global I/O reactor.
pub fn global_reactor() -> &'static IoReactor {
    unsafe {
        GLOBAL_REACTOR
            .as_ref()
            .expect("I/O reactor not initialized")
    }
}

/// A dummy I/O backend for testing or initial development.
///
/// This backend doesn't actually perform any async I/O, but it allows
/// the code to compile and run in a synchronous manner.
#[derive(Default)]
struct DummyBackend {
    completions: Mutex<VecDeque<(RequestId, Completion)>>,
}

impl DummyBackend {
    fn new() -> Self {
        Self {
            completions: Mutex::new(VecDeque::new()),
        }
    }
}

impl AsyncIoBackend for DummyBackend {
    fn submit(&self, id: RequestId, operation: IoOperation) {
        let completion = match operation {
            IoOperation::Read { socket, buf, len } => {
                // Cast socket to its concrete type
                if let Ok(socket) = socket.downcast::<Mutex<axnet::TcpSocket>>() {
                    let mut socket = socket.lock();
                    let ptr = buf as *mut u8;
                    let slice = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
                    match socket.recv(slice) {
                        Ok(n) => Completion::Read(n),
                        Err(e) => Completion::Error(Error::from(e)),
                    }
                } else if let Ok(socket) = socket.downcast::<Mutex<axnet::UdpSocket>>() {
                    let mut socket = socket.lock();
                    let ptr = buf as *mut u8;
                    let slice = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
                    match socket.recv(slice) {
                        Ok(n) => Completion::Read(n),
                        Err(e) => Completion::Error(Error::from(e)),
                    }
                } else {
                    Completion::Error(Error::new(
                        super::ErrorKind::InvalidInput,
                        "Unknown socket type".into(),
                    ))
                }
            }
            IoOperation::Write { socket, buf, len } => {
                // Cast socket to its concrete type
                if let Ok(socket) = socket.downcast::<Mutex<axnet::TcpSocket>>() {
                    let mut socket = socket.lock();
                    let ptr = buf as *const u8;
                    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
                    match socket.send(slice) {
                        Ok(n) => Completion::Write(n),
                        Err(e) => Completion::Error(Error::from(e)),
                    }
                } else if let Ok(socket) = socket.downcast::<Mutex<axnet::UdpSocket>>() {
                    let mut socket = socket.lock();
                    let ptr = buf as *const u8;
                    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
                    match socket.send(slice) {
                        Ok(n) => Completion::Write(n),
                        Err(e) => Completion::Error(Error::from(e)),
                    }
                } else {
                    Completion::Error(Error::new(
                        super::ErrorKind::InvalidInput,
                        "Unknown socket type".into(),
                    ))
                }
            }
            IoOperation::Connect { socket, addr } => {
                if let Ok(socket) = socket.downcast::<Mutex<axnet::TcpSocket>>() {
                    let mut socket = socket.lock();
                    match socket.connect(addr) {
                        Ok(_) => Completion::Connect,
                        Err(e) => Completion::Error(Error::from(e)),
                    }
                } else {
                    Completion::Error(Error::new(
                        super::ErrorKind::InvalidInput,
                        "Socket is not a TCP socket".into(),
                    ))
                }
            }
            IoOperation::Accept { socket } => {
                if let Ok(socket) = socket.downcast::<Mutex<axnet::TcpSocket>>() {
                    let mut socket = socket.lock();
                    match socket.accept() {
                        Ok(new_socket) => Completion::Accept(Box::new(new_socket)),
                        Err(e) => Completion::Error(Error::from(e)),
                    }
                } else {
                    Completion::Error(Error::new(
                        super::ErrorKind::InvalidInput,
                        "Socket is not a TCP socket".into(),
                    ))
                }
            }
            IoOperation::Send { socket, buf, len } => {
                if let Ok(socket) = socket.downcast::<Mutex<axnet::UdpSocket>>() {
                    let mut socket = socket.lock();
                    let ptr = buf as *const u8;
                    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
                    match socket.send(slice) {
                        Ok(n) => Completion::Send(n),
                        Err(e) => Completion::Error(Error::from(e)),
                    }
                } else {
                    Completion::Error(Error::new(
                        super::ErrorKind::InvalidInput,
                        "Socket is not a UDP socket".into(),
                    ))
                }
            }
            IoOperation::SendTo {
                socket,
                buf,
                len,
                addr,
            } => {
                if let Ok(socket) = socket.downcast::<Mutex<axnet::UdpSocket>>() {
                    let mut socket = socket.lock();
                    let ptr = buf as *const u8;
                    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
                    match socket.send_to(slice, addr) {
                        Ok(n) => Completion::SendTo(n),
                        Err(e) => Completion::Error(Error::from(e)),
                    }
                } else {
                    Completion::Error(Error::new(
                        super::ErrorKind::InvalidInput,
                        "Socket is not a UDP socket".into(),
                    ))
                }
            }
            IoOperation::Recv { socket, buf, len } => {
                if let Ok(socket) = socket.downcast::<Mutex<axnet::UdpSocket>>() {
                    let mut socket = socket.lock();
                    let ptr = buf as *mut u8;
                    let slice = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
                    match socket.recv(slice) {
                        Ok(n) => Completion::Recv(n),
                        Err(e) => Completion::Error(Error::from(e)),
                    }
                } else {
                    Completion::Error(Error::new(
                        super::ErrorKind::InvalidInput,
                        "Socket is not a UDP socket".into(),
                    ))
                }
            }
            IoOperation::RecvFrom { socket, buf, len } => {
                if let Ok(socket) = socket.downcast::<Mutex<axnet::UdpSocket>>() {
                    let mut socket = socket.lock();
                    let ptr = buf as *mut u8;
                    let slice = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
                    match socket.recv_from(slice) {
                        Ok((n, addr)) => Completion::RecvFrom(n, addr),
                        Err(e) => Completion::Error(Error::from(e)),
                    }
                } else {
                    Completion::Error(Error::new(
                        super::ErrorKind::InvalidInput,
                        "Socket is not a UDP socket".into(),
                    ))
                }
            }
        };

        self.completions.lock().push_back((id, completion));
    }

    fn poll(&self) -> Vec<(RequestId, Completion)> {
        let mut completions = self.completions.lock();
        let result: Vec<_> = completions.drain(..).collect();
        result
    }
}
