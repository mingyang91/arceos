# ArceOS Async I/O Module

This module provides asynchronous I/O operations for ArceOS, including file and network I/O.

## Architecture

The async I/O system is built around a reactor pattern, which consists of:

1. **I/O Reactor**: Manages I/O operations and delivers completions to waiting futures
2. **I/O Operations**: Represent non-blocking operations like read, write, connect, etc.
3. **I/O Futures**: Represent pending operations that will complete in the future
4. **AsyncRead/AsyncWrite Traits**: Foundational traits for async I/O operations

### Component Diagram

```
                   ┌─────────────┐
                   │   axasync   │
                   └──────┬──────┘
                          │
              ┌───────────┴───────────┐
              │                       │
       ┌──────▼─────┐          ┌──────▼─────┐
       │  io::file  │          │  io::net   │
       └──────┬─────┘          └──────┬─────┘
              │                       │
              │                       │
       ┌──────▼─────┐          ┌──────▼─────┐
       │   axfs     │          │   axnet    │
       └────────────┘          └────────────┘
```

## Core Traits

### AsyncRead

Provides asynchronous reading of data into buffers:

```rust
pub trait AsyncRead {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8]
    ) -> Poll<Result<usize>>;
    
    // Additional default methods...
}
```

### AsyncWrite

Provides asynchronous writing of data from buffers:

```rust
pub trait AsyncWrite {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8]
    ) -> Poll<Result<usize>>;
    
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Result<()>>;
    
    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Result<()>>;
    
    // Additional default methods...
}
```

## I/O Reactor

The I/O reactor is responsible for:

1. Submitting I/O operations to the backend
2. Polling for completed operations
3. Waking up futures when their operations complete

It uses an `AsyncIoBackend` implementation to handle the low-level details of I/O operations. A default implementation is provided for synchronous I/O operations, but more efficient backends can be implemented for specific platforms or use cases.

## Networking

The networking module provides async wrappers around ArceOS's synchronous network stack:

- `TcpSocket`: Async TCP socket implementing `AsyncRead` and `AsyncWrite`
- `UdpSocket`: Async UDP socket for datagram-based networking
- Extension traits for common operations like connection establishment

## Buffer Types

For improved performance, the module includes buffered implementations:

- `BufReader<R>`: Buffered reader for more efficient reading
- `BufWriter<W>`: Buffered writer for more efficient writing

## Usage Examples

### TCP Client

```rust
use axasync::{TcpSocket, AsyncReadExt, AsyncWriteExt};

async fn connect_and_send() -> Result<()> {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let mut socket = TcpSocket::connect_to(addr).await?;
    
    socket.write_all(b"Hello, world!").await?;
    
    let mut buf = [0u8; 128];
    let n = socket.read(&mut buf).await?;
    println!("Received: {}", core::str::from_utf8(&buf[..n])?);
    
    socket.close().await?;
    Ok(())
}
```

### TCP Server

```rust
use axasync::{TcpSocket, AsyncReadExt, AsyncWriteExt};

async fn run_server() -> Result<()> {
    let addr = "0.0.0.0:8080".parse().unwrap();
    let socket = TcpSocket::new();
    socket.bind(addr)?;
    socket.listen()?;
    
    while let Ok(mut client) = socket.accept().await {
        // Handle each client in a separate task
        axasync::spawn(async move {
            let mut buf = [0u8; 1024];
            while let Ok(n) = client.read(&mut buf).await {
                if n == 0 { break; }
                client.write_all(&buf[..n]).await.unwrap();
            }
        });
    }
    
    Ok(())
}
```

## Error Handling

The module provides a comprehensive error type that can represent various I/O errors, with conversions from the underlying axerrno error types.

## Implementation Notes

- The current implementation uses a synchronous I/O backend, which means I/O operations still block, but at a higher level of abstraction.
- Future improvements could include integrating with interrupt-driven I/O or true non-blocking I/O APIs when they become available in ArceOS.
- The reactor design allows for future optimizations without changing the API surface. 