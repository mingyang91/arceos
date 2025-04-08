# Async TCP Client Example

This example demonstrates a simple asynchronous TCP client using ArceOS's async runtime. It connects to a TCP server, sends a message, and processes the response.

## Features

- Asynchronous TCP socket connection
- Sending and receiving data over TCP
- Error handling

## Usage

To run the async TCP client:

```bash
# Run with networking enabled
make A=examples/async_client ARCH=riscv64 LOG=info NET=y run
```

By default, the client attempts to connect to `10.0.2.2:8000`, which corresponds to the host machine in QEMU's user networking mode.

## Network Configuration

The client is configured to connect to the host machine running QEMU. If you want to connect to a different server, you can modify the server address in the source code.

## Integration with Server

This client works with the `async_server` example. To test them together:

1. Start the server on your host machine first (if using a host-based server)
2. Run this client in QEMU, which will connect to the host

If testing the server example within another QEMU instance, you'll need to configure appropriate network settings to allow connections between VMs. 