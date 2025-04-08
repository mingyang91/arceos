# Async TCP Server Example

This example demonstrates a simple asynchronous TCP server using ArceOS's async runtime. It accepts incoming connections, processes client messages, and sends responses.

## Features

- Asynchronous TCP socket server
- Handling multiple client connections
- Echo-style response with message enhancement

## Usage

To run the async TCP server:

```bash
# Run with networking enabled and port forwarding
make A=examples/async_server ARCH=riscv64 LOG=info NET=y QEMU_OPTS="-serial mon:stdio -device virtio-net-device,netdev=net0 -netdev user,id=net0,hostfwd=tcp::8000-:8000" run
```

The server listens on all interfaces (0.0.0.0) on port 8000.

## Network Configuration

The server binds to all network interfaces on port 8000. When running in QEMU, you'll need to forward the host port to the guest using the `hostfwd` option as shown in the usage example.

## Client Communication

When a client connects and sends a message, the server:

1. Receives and logs the message
2. Processes the message
3. Responds with "Server received: [original message]"
4. Closes the connection

## Integration with Client

This server works with the `async_client` example. To test them together:

1. Start the server first with port forwarding enabled
2. Run the client, which will connect to the server

The server will continue running and accepting new connections until explicitly stopped. 