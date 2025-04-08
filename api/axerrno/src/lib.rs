//! Error types for ArceOS.
//!
//! This crate provides common error types used throughout the ArceOS ecosystem.

#![no_std]

use core::fmt;

/// A specialized [`Result`] type for ArceOS operations.
///
/// This type is used throughout the ArceOS codebase to unify error handling.
pub type AxResult<T = ()> = Result<T, AxError>;

/// A Linux-style result type.
pub type LinuxResult<T = ()> = Result<T, LinuxError>;

/// ArceOS common error codes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
#[non_exhaustive]
pub enum AxError {
    /// Operation not permitted
    PermissionDenied = 1,
    /// No such file or directory
    NotFound = 2,
    /// No such process
    NoProcess = 3,
    /// Interrupted system call
    Interrupted = 4,
    /// I/O error
    IoError = 5,
    /// No such device or address
    NoDevice = 6,
    /// Arg list too long
    ArgListTooLong = 7,
    /// Exec format error
    ExecFormatError = 8,
    /// Bad file number
    BadFileNumber = 9,
    /// No child processes
    NoChildProcess = 10,
    /// Try again
    Again = 11,
    /// Out of memory
    NoMemory = 12,
    /// Permission denied
    PermDenied = 13,
    /// Bad address
    BadAddress = 14,
    /// Block device required
    BlockDeviceRequired = 15,
    /// Device or resource busy
    Busy = 16,
    /// File exists
    AlreadyExists = 17,
    /// Cross-device link
    CrossDeviceLink = 18,
    /// No such device
    NoSuchDevice = 19,
    /// Not a directory
    NotADirectory = 20,
    /// Is a directory
    IsADirectory = 21,
    /// Invalid argument
    InvalidInput = 22,
    /// File table overflow
    FileTableOverflow = 23,
    /// Too many open files
    TooManyOpenFiles = 24,
    /// Not a typewriter
    NotATty = 25,
    /// Text file busy
    TextFileBusy = 26,
    /// File too large
    FileTooLarge = 27,
    /// No space left on device
    NoSpaceLeftOnDevice = 28,
    /// Illegal seek
    IllegalSeek = 29,
    /// Read-only file system
    ReadOnlyFileSystem = 30,
    /// Too many links
    TooManyLinks = 31,
    /// Broken pipe
    BrokenPipe = 32,
    /// Math argument out of domain of func
    MathOutOfDomain = 33,
    /// Math result not representable
    MathNotRepresentable = 34,
    /// Function not implemented
    NotImplemented = 35,
    /// Block IO error
    BlockIoError = 36,
    /// Non-existant mapping
    NonExistantMapping = 37,
    /// Timer expired
    TimedOut = 38,
    /// Connection refused
    ConnectionRefused = 39,
    /// Connection aborted
    ConnectionAborted = 40,
    /// Connection already in progress
    ConnectionInProgress = 41,
    /// Connection timed out
    ConnectionTimedOut = 42,
    /// Connection is already connected
    AlreadyConnected = 43,
    /// Connection was reset
    ConnectionReset = 44,
    /// Connection is not connected
    NotConnected = 45,
    /// Address already in use
    AddrInUse = 46,
    /// Address not available
    AddrNotAvailable = 47,
    /// Network is down
    NetworkDown = 48,
    /// Network is unreachable
    NetworkUnreachable = 49,
    /// Network dropped connection because of reset
    NetworkReset = 50,
    /// Software caused connection abort
    SoftwareConnectionAbort = 51,
    /// Operation would block
    WouldBlock = 52,
    /// Operation already in progress
    InProgress = 53,
    /// Operation not supported
    Unsupported = 54,
    /// Protocol family not supported
    ProtocolFamilyNotSupported = 55,
    /// Protocol not supported
    ProtocolNotSupported = 56,
    /// Protocol wrong type for socket
    ProtocolWrongType = 57,
    /// Invalid memory range
    InvalidMemRange = 58,
    /// Destination address required
    DestinationAddressRequired = 59,
    /// Message too large
    MessageTooLarge = 60,
    /// Protocol wrong type for socket
    WrongProtocolType = 61,
    /// Protocol not available
    ProtocolNotAvailable = 62,
    /// Unknown protocol
    UnknownProtocol = 63,
    /// Socket operation on non-socket
    NotASocket = 64,
    /// Protocol family not supported
    AddressFamilyNotSupported = 65,
    /// Socket type not supported
    SocketTypeNotSupported = 66,
    /// Connection reset by peer
    ConnectionResetByPeer = 67,
    /// Transport endpoint is already connected
    TransportEndpointAlreadyConnected = 68,
    /// Transport endpoint is not connected
    TransportEndpointNotConnected = 69,
    /// Hostname lookup failed
    HostLookupFailed = 70,
    /// Operation not supported on transport endpoint
    OperationNotSupportedOnEndpoint = 71,
    /// Socket is shut down
    SocketShutdown = 72,
    /// Disk error
    DiskError = 73,
}

impl fmt::Display for AxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::PermissionDenied => "permission denied",
            Self::NotFound => "not found",
            Self::NoProcess => "no such process",
            Self::Interrupted => "interrupted",
            Self::IoError => "I/O error",
            Self::NoDevice => "no such device or address",
            Self::ArgListTooLong => "argument list too long",
            Self::ExecFormatError => "exec format error",
            Self::BadFileNumber => "bad file number",
            Self::NoChildProcess => "no child processes",
            Self::Again => "try again",
            Self::NoMemory => "out of memory",
            Self::PermDenied => "permission denied",
            Self::BadAddress => "bad address",
            Self::BlockDeviceRequired => "block device required",
            Self::Busy => "device or resource busy",
            Self::AlreadyExists => "file exists",
            Self::CrossDeviceLink => "cross-device link",
            Self::NoSuchDevice => "no such device",
            Self::NotADirectory => "not a directory",
            Self::IsADirectory => "is a directory",
            Self::InvalidInput => "invalid argument",
            Self::FileTableOverflow => "file table overflow",
            Self::TooManyOpenFiles => "too many open files",
            Self::NotATty => "not a typewriter",
            Self::TextFileBusy => "text file busy",
            Self::FileTooLarge => "file too large",
            Self::NoSpaceLeftOnDevice => "no space left on device",
            Self::IllegalSeek => "illegal seek",
            Self::ReadOnlyFileSystem => "read-only file system",
            Self::TooManyLinks => "too many links",
            Self::BrokenPipe => "broken pipe",
            Self::MathOutOfDomain => "math argument out of domain of func",
            Self::MathNotRepresentable => "math result not representable",
            Self::NotImplemented => "function not implemented",
            Self::BlockIoError => "block I/O error",
            Self::NonExistantMapping => "non-existent mapping",
            Self::TimedOut => "timer expired",
            Self::ConnectionRefused => "connection refused",
            Self::ConnectionAborted => "connection aborted",
            Self::ConnectionInProgress => "connection already in progress",
            Self::ConnectionTimedOut => "connection timed out",
            Self::AlreadyConnected => "connection is already connected",
            Self::ConnectionReset => "connection was reset",
            Self::NotConnected => "connection is not connected",
            Self::AddrInUse => "address already in use",
            Self::AddrNotAvailable => "address not available",
            Self::NetworkDown => "network is down",
            Self::NetworkUnreachable => "network is unreachable",
            Self::NetworkReset => "network dropped connection because of reset",
            Self::SoftwareConnectionAbort => "software caused connection abort",
            Self::WouldBlock => "operation would block",
            Self::InProgress => "operation already in progress",
            Self::Unsupported => "operation not supported",
            Self::ProtocolFamilyNotSupported => "protocol family not supported",
            Self::ProtocolNotSupported => "protocol not supported",
            Self::ProtocolWrongType => "protocol wrong type for socket",
            Self::InvalidMemRange => "invalid memory range",
            Self::DestinationAddressRequired => "destination address required",
            Self::MessageTooLarge => "message too large",
            Self::WrongProtocolType => "protocol wrong type for socket",
            Self::ProtocolNotAvailable => "protocol not available",
            Self::UnknownProtocol => "unknown protocol",
            Self::NotASocket => "socket operation on non-socket",
            Self::AddressFamilyNotSupported => "protocol family not supported",
            Self::SocketTypeNotSupported => "socket type not supported",
            Self::ConnectionResetByPeer => "connection reset by peer",
            Self::TransportEndpointAlreadyConnected => "transport endpoint is already connected",
            Self::TransportEndpointNotConnected => "transport endpoint is not connected",
            Self::HostLookupFailed => "hostname lookup failed",
            Self::OperationNotSupportedOnEndpoint =>
                "operation not supported on transport endpoint",
            Self::SocketShutdown => "socket is shut down",
            Self::DiskError => "disk error",
        })
    }
}

/// Linux error codes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
#[non_exhaustive]
pub enum LinuxError {
    EPERM = 1,            /* Operation not permitted */
    ENOENT = 2,           /* No such file or directory */
    ESRCH = 3,            /* No such process */
    EINTR = 4,            /* Interrupted system call */
    EIO = 5,              /* I/O error */
    ENXIO = 6,            /* No such device or address */
    E2BIG = 7,            /* Argument list too long */
    ENOEXEC = 8,          /* Exec format error */
    EBADF = 9,            /* Bad file number */
    ECHILD = 10,          /* No child processes */
    EAGAIN = 11,          /* Try again */
    ENOMEM = 12,          /* Out of memory */
    EACCES = 13,          /* Permission denied */
    EFAULT = 14,          /* Bad address */
    ENOTBLK = 15,         /* Block device required */
    EBUSY = 16,           /* Device or resource busy */
    EEXIST = 17,          /* File exists */
    EXDEV = 18,           /* Cross-device link */
    ENODEV = 19,          /* No such device */
    ENOTDIR = 20,         /* Not a directory */
    EISDIR = 21,          /* Is a directory */
    EINVAL = 22,          /* Invalid argument */
    ENFILE = 23,          /* File table overflow */
    EMFILE = 24,          /* Too many open files */
    ENOTTY = 25,          /* Not a typewriter */
    ETXTBSY = 26,         /* Text file busy */
    EFBIG = 27,           /* File too large */
    ENOSPC = 28,          /* No space left on device */
    ESPIPE = 29,          /* Illegal seek */
    EROFS = 30,           /* Read-only file system */
    EMLINK = 31,          /* Too many links */
    EPIPE = 32,           /* Broken pipe */
    EDOM = 33,            /* Math argument out of domain of func */
    ERANGE = 34,          /* Math result not representable */
    ENOSYS = 35,          /* Function not implemented */
    ELOOP = 36,           /* Too many symbolic links encountered */
    ENAMETOOLONG = 37,    /* File name too long */
    EBADFD = 38,          /* File descriptor in bad state */
    EADDRINUSE = 39,      /* Address already in use */
    EADDRNOTAVAIL = 40,   /* Cannot assign requested address */
    ENETDOWN = 41,        /* Network is down */
    ENETUNREACH = 42,     /* Network is unreachable */
    ENETRESET = 43,       /* Network dropped connection because of reset */
    ECONNRESET = 44,      /* Connection reset by peer */
    ENOBUFS = 45,         /* No buffer space available */
    EISCONN = 46,         /* Transport endpoint is already connected */
    ENOTCONN = 47,        /* Transport endpoint is not connected */
    ETIMEDOUT = 48,       /* Connection timed out */
    ECONNREFUSED = 49,    /* Connection refused */
    EHOSTUNREACH = 50,    /* No route to host */
    EALREADY = 51,        /* Operation already in progress */
    EINPROGRESS = 52,     /* Operation now in progress */
    EWOULDBLOCK = 53,     /* Operation would block */
    ENOTSOCK = 54,        /* Socket operation on non-socket */
    EMSGSIZE = 55,        /* Message too long */
    EPROTOTYPE = 56,      /* Protocol wrong type for socket */
    ENOPROTOOPT = 57,     /* Protocol not available */
    EPROTONOSUPPORT = 58, /* Protocol not supported */
    EAFNOSUPPORT = 59,    /* Address family not supported by protocol */
    ENOTSUP = 60,         /* Operation not supported on transport endpoint */
    ENOSYS2 = 61,         /* Function not implemented */
    EPROTO = 62,          /* Protocol error */
    EOVERFLOW = 63,       /* Value too large for defined data type */
    EBADMSG = 64,         /* Not a data message */
}

impl LinuxError {
    /// Returns the corresponding error code.
    #[inline]
    pub const fn code(self) -> i32 {
        self as i32
    }

    /// Returns the corresponding error string.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EPERM => "Operation not permitted",
            Self::ENOENT => "No such file or directory",
            Self::ESRCH => "No such process",
            Self::EINTR => "Interrupted system call",
            Self::EIO => "I/O error",
            Self::ENXIO => "No such device or address",
            Self::E2BIG => "Argument list too long",
            Self::ENOEXEC => "Exec format error",
            Self::EBADF => "Bad file number",
            Self::ECHILD => "No child processes",
            Self::EAGAIN => "Try again",
            Self::ENOMEM => "Out of memory",
            Self::EACCES => "Permission denied",
            Self::EFAULT => "Bad address",
            Self::ENOTBLK => "Block device required",
            Self::EBUSY => "Device or resource busy",
            Self::EEXIST => "File exists",
            Self::EXDEV => "Cross-device link",
            Self::ENODEV => "No such device",
            Self::ENOTDIR => "Not a directory",
            Self::EISDIR => "Is a directory",
            Self::EINVAL => "Invalid argument",
            Self::ENFILE => "File table overflow",
            Self::EMFILE => "Too many open files",
            Self::ENOTTY => "Not a typewriter",
            Self::ETXTBSY => "Text file busy",
            Self::EFBIG => "File too large",
            Self::ENOSPC => "No space left on device",
            Self::ESPIPE => "Illegal seek",
            Self::EROFS => "Read-only file system",
            Self::EMLINK => "Too many links",
            Self::EPIPE => "Broken pipe",
            Self::EDOM => "Math argument out of domain of func",
            Self::ERANGE => "Math result not representable",
            Self::ENOSYS => "Function not implemented",
            Self::ELOOP => "Too many symbolic links encountered",
            Self::ENAMETOOLONG => "File name too long",
            Self::EBADFD => "File descriptor in bad state",
            Self::EADDRINUSE => "Address already in use",
            Self::EADDRNOTAVAIL => "Cannot assign requested address",
            Self::ENETDOWN => "Network is down",
            Self::ENETUNREACH => "Network is unreachable",
            Self::ENETRESET => "Network dropped connection because of reset",
            Self::ECONNRESET => "Connection reset by peer",
            Self::ENOBUFS => "No buffer space available",
            Self::EISCONN => "Transport endpoint is already connected",
            Self::ENOTCONN => "Transport endpoint is not connected",
            Self::ETIMEDOUT => "Connection timed out",
            Self::ECONNREFUSED => "Connection refused",
            Self::EHOSTUNREACH => "No route to host",
            Self::EALREADY => "Operation already in progress",
            Self::EINPROGRESS => "Operation now in progress",
            Self::EWOULDBLOCK => "Operation would block",
            Self::ENOTSOCK => "Socket operation on non-socket",
            Self::EMSGSIZE => "Message too long",
            Self::EPROTOTYPE => "Protocol wrong type for socket",
            Self::ENOPROTOOPT => "Protocol not available",
            Self::EPROTONOSUPPORT => "Protocol not supported",
            Self::EAFNOSUPPORT => "Address family not supported by protocol",
            Self::ENOTSUP => "Operation not supported on transport endpoint",
            Self::ENOSYS2 => "Function not implemented",
            Self::EPROTO => "Protocol error",
            Self::EOVERFLOW => "Value too large for defined data type",
            Self::EBADMSG => "Not a data message",
        }
    }
}

impl TryFrom<i32> for LinuxError {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value >= 1 && value <= 64 {
            // SAFETY: We checked the range and the enum has that many variants
            Ok(unsafe { core::mem::transmute(value) })
        } else {
            Err(())
        }
    }
}

/// Creates a new AxError with the specified type and message.
///
/// # Examples
///
/// ```
/// use axerrno::{ax_err, AxError};
///
/// let err = ax_err!(NotFound, "file not found");
/// assert_eq!(err, Err(AxError::NotFound));
/// ```
#[macro_export]
macro_rules! ax_err {
    ($err_type:ident, $msg:expr) => {
        Err($crate::AxError::$err_type)
    };
}

/// Creates a new AxError with the specified type and returns it immediately.
///
/// # Examples
///
/// ```
/// use axerrno::{ax_err_type, AxError};
///
/// fn may_fail() -> axerrno::AxResult<()> {
///     ax_err_type!(NotFound, "file not found")
/// }
/// ```
#[macro_export]
macro_rules! ax_err_type {
    ($err_type:ident, $msg:expr) => {
        return Err($crate::AxError::$err_type)
    };
}
