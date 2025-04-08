#![no_std]
#![no_main]

extern crate alloc;

use axasync::{block_on, init, shutdown, AsyncRead, AsyncWrite, TcpSocket, TcpSocketExt};
use axstd::println;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};

#[no_mangle]
fn main() {
    // Initialize the async runtime
    init();

    println!("Async TCP Client");

    // Connect to a TCP server
    let result = block_on(run_client());
    match result {
        Ok(_) => println!("Client completed successfully"),
        Err(e) => println!("Client error: {}", e),
    }

    // Shutdown the async runtime
    shutdown();
}

/// The main client function that connects to a server and exchanges messages
async fn run_client() -> Result<(), &'static str> {
    // By default, connect to 10.0.2.2:8000 which is the host machine in QEMU's user networking mode
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 2)), 8000);

    println!("Connecting to server at {}...", server_addr);

    let mut socket = TcpSocket::connect_to(server_addr)
        .await
        .map_err(|_| "Failed to connect to server")?;

    println!("Connected to server!");

    // Send a message to the server
    let message = b"Hello from ArceOS async client!";
    socket
        .write_all(message)
        .await
        .map_err(|_| "Failed to send message")?;

    println!("Sent message: {}", core::str::from_utf8(message).unwrap());

    // Read the response from the server
    let mut buf = [0u8; 1024];
    let n = socket
        .read(&mut buf)
        .await
        .map_err(|_| "Failed to read response")?;

    if n == 0 {
        println!("Server closed the connection");
        return Ok(());
    }

    // Process and display the response
    let response = core::str::from_utf8(&buf[..n]).unwrap_or("Invalid UTF-8 response");
    println!("Received from server: {}", response);

    // Close the connection
    socket
        .close()
        .await
        .map_err(|_| "Failed to close connection")?;

    println!("Connection closed");

    Ok(())
}
