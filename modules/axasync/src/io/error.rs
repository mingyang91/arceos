//! Error types for async I/O operations

use alloc::format;
use alloc::string::String;
use axerrno::AxError;
use core::fmt;

/// I/O error kind.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An entity was not found.
    NotFound,
    /// The operation lacked the necessary privileges.
    PermissionDenied,
    /// The connection was refused by the remote server.
    ConnectionRefused,
    /// The connection was reset by the remote server.
    ConnectionReset,
    /// The connection was aborted by the remote server.
    ConnectionAborted,
    /// The network operation failed because it was not connected yet.
    NotConnected,
    /// A socket address could not be bound because the address is already in use.
    AddrInUse,
    /// A nonexistent interface was requested or the requested address was not local.
    AddrNotAvailable,
    /// The operation failed because a pipe was closed.
    BrokenPipe,
    /// An entity already exists.
    AlreadyExists,
    /// The operation needs to block to complete, but the blocking operation was requested to not occur.
    WouldBlock,
    /// A parameter was incorrect.
    InvalidInput,
    /// Data not valid for the operation were encountered.
    InvalidData,
    /// The I/O operation's timeout expired.
    TimedOut,
    /// The write operation failed because it would cause the write buffer to exceed available space.
    WriteZero,
    /// An error returned when an operation could not be completed because a call to write returned Ok(0).
    ReadZero,
    /// An error returned when an operation could not be completed because an underlying file descriptor was closed.
    Disconnected,
    /// This operation was interrupted.
    Interrupted,
    /// Any I/O error not part of this list.
    Other,
    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    UnexpectedEof,
    /// An operation could not be completed because there was not enough storage space.
    OutOfMemory,
}

impl ErrorKind {
    /// Returns a static string description of the error.
    pub fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::NotFound => "entity not found",
            ErrorKind::PermissionDenied => "permission denied",
            ErrorKind::ConnectionRefused => "connection refused",
            ErrorKind::ConnectionReset => "connection reset",
            ErrorKind::ConnectionAborted => "connection aborted",
            ErrorKind::NotConnected => "not connected",
            ErrorKind::AddrInUse => "address in use",
            ErrorKind::AddrNotAvailable => "address not available",
            ErrorKind::BrokenPipe => "broken pipe",
            ErrorKind::AlreadyExists => "entity already exists",
            ErrorKind::WouldBlock => "operation would block",
            ErrorKind::InvalidInput => "invalid input parameter",
            ErrorKind::InvalidData => "invalid data",
            ErrorKind::TimedOut => "timed out",
            ErrorKind::WriteZero => "write zero",
            ErrorKind::ReadZero => "read zero",
            ErrorKind::Disconnected => "disconnected",
            ErrorKind::Interrupted => "operation interrupted",
            ErrorKind::Other => "other I/O error",
            ErrorKind::UnexpectedEof => "unexpected end of file",
            ErrorKind::OutOfMemory => "out of memory",
        }
    }
}

/// The error type for I/O operations.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    /// Creates a new I/O error from the specified kind and message.
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { kind, message }
    }

    /// Returns the error kind.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Creates a new I/O error for unexpected EOF.
    pub fn unexpected_eof() -> Self {
        Self::new(ErrorKind::UnexpectedEof, "unexpected end of file".into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind.as_str(), self.message)
    }
}

impl From<AxError> for Error {
    fn from(error: AxError) -> Self {
        let kind = match error {
            AxError::NotFound => ErrorKind::NotFound,
            AxError::PermissionDenied | AxError::PermDenied => ErrorKind::PermissionDenied,
            AxError::ConnectionRefused => ErrorKind::ConnectionRefused,
            AxError::ConnectionReset | AxError::ConnectionResetByPeer => ErrorKind::ConnectionReset,
            AxError::ConnectionAborted => ErrorKind::ConnectionAborted,
            AxError::NotConnected | AxError::TransportEndpointNotConnected => {
                ErrorKind::NotConnected
            }
            AxError::AddrInUse => ErrorKind::AddrInUse,
            AxError::AddrNotAvailable => ErrorKind::AddrNotAvailable,
            AxError::BrokenPipe => ErrorKind::BrokenPipe,
            AxError::AlreadyExists => ErrorKind::AlreadyExists,
            AxError::WouldBlock => ErrorKind::WouldBlock,
            AxError::InvalidInput => ErrorKind::InvalidInput,
            AxError::TimedOut | AxError::ConnectionTimedOut => ErrorKind::TimedOut,
            AxError::NoMemory => ErrorKind::OutOfMemory,
            AxError::Interrupted => ErrorKind::Interrupted,
            _ => ErrorKind::Other,
        };
        Self::new(kind, format!("{}", error))
    }
}

impl From<axerrno::LinuxError> for Error {
    fn from(error: axerrno::LinuxError) -> Self {
        let kind = match error {
            axerrno::LinuxError::ENOENT => ErrorKind::NotFound,
            axerrno::LinuxError::EPERM | axerrno::LinuxError::EACCES => ErrorKind::PermissionDenied,
            axerrno::LinuxError::ECONNREFUSED => ErrorKind::ConnectionRefused,
            axerrno::LinuxError::ECONNRESET => ErrorKind::ConnectionReset,
            axerrno::LinuxError::ENOTCONN => ErrorKind::NotConnected,
            axerrno::LinuxError::EADDRINUSE => ErrorKind::AddrInUse,
            axerrno::LinuxError::EADDRNOTAVAIL => ErrorKind::AddrNotAvailable,
            axerrno::LinuxError::EPIPE => ErrorKind::BrokenPipe,
            axerrno::LinuxError::EEXIST => ErrorKind::AlreadyExists,
            axerrno::LinuxError::EWOULDBLOCK | axerrno::LinuxError::EAGAIN => ErrorKind::WouldBlock,
            axerrno::LinuxError::EINVAL => ErrorKind::InvalidInput,
            axerrno::LinuxError::ETIMEDOUT => ErrorKind::TimedOut,
            axerrno::LinuxError::ENOMEM => ErrorKind::OutOfMemory,
            axerrno::LinuxError::EINTR => ErrorKind::Interrupted,
            _ => ErrorKind::Other,
        };
        Self::new(kind, format!("Linux error: {}", error.as_str()))
    }
}

/// A specialized Result type for I/O operations.
pub type Result<T> = core::result::Result<T, Error>;
