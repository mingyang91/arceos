# Async Networking Demo

This example demonstrates the use of the asynchronous networking API in ArceOS.

## Description

The example includes both a client and server implementation:

- The client connects to a server, sends a message, and waits for a response.
- The server listens for incoming connections and echoes back any data it receives.

The networking API uses futures and `async`/`await` syntax for a clean, non-blocking programming model.

## Features

- Asynchronous TCP connection establishment
- Async read and write operations
- Proper resource cleanup with close operations
- Integration with the async runtime

## Running

To run the server:

```bash
make A=examples/async_net_demo ARCH=x86_64 LOG=info NET=y SMP=1 run SERVER=y
```

To run the client:

```bash
make A=examples/async_net_demo ARCH=x86_64 LOG=info NET=y SMP=1 run
```

Note: You'll need to run the server first in one terminal, and then the client in another terminal.

## Implementation Details

The implementation demonstrates how to:

1. Initialize the async runtime with `axasync::init()`
2. Create and use async TCP sockets
3. Perform non-blocking I/O operations
4. Use `AsyncReadExt` and `AsyncWriteExt` traits for convenience methods
5. Handle errors in an async context
6. Clean up resources when done

It builds upon the reactor-based design of ArceOS's async I/O system, which provides a future-based interface on top of the existing synchronous networking stack. 