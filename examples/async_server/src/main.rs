#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use axasync::{block_on, init, shutdown, AsyncRead, AsyncWrite, TcpSocket};
use axstd::println;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};

#[no_mangle]
fn main() {
    // Initialize the async runtime
    init();

    println!("Async TCP Server");

    // Start the TCP server
    let result = block_on(run_server());
    match result {
        Ok(_) => println!("Server completed successfully"),
        Err(e) => println!("Server error: {}", e),
    }

    // Shutdown the async runtime
    shutdown();
}

/// The main server function that accepts connections and handles client requests
async fn run_server() -> Result<(), &'static str> {
    // Listen on all interfaces on port 8000
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8000);

    let socket = TcpSocket::new();
    socket.bind(addr).map_err(|_| "Failed to bind to address")?;
    socket.listen().map_err(|_| "Failed to listen")?;

    println!("Server listening on {}", addr);

    // Accept and handle client connections
    loop {
        match socket.accept().await {
            Ok(mut client) => {
                let peer_addr = client
                    .peer_addr()
                    .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0));
                println!("Client connected from {}", peer_addr);

                // Handle client connection
                handle_client(&mut client).await?;

                println!("Client disconnected: {}", peer_addr);
            }
            Err(e) => {
                println!("Failed to accept connection: {:?}", e);
                // Continue accepting other connections
            }
        }
    }
}

/// Handle a client connection by reading data and sending responses
async fn handle_client(client: &mut (impl AsyncRead + AsyncWrite)) -> Result<(), &'static str> {
    let mut buffer = [0u8; 1024];

    // Read data from client
    let bytes_read = client
        .read(&mut buffer)
        .await
        .map_err(|_| "Failed to read from client")?;

    if bytes_read == 0 {
        // Client closed the connection
        return Ok(());
    }

    // Process the received data
    let message = core::str::from_utf8(&buffer[..bytes_read]).unwrap_or("Invalid UTF-8");
    println!("Received message: {}", message);

    // Prepare and send response (echo the received message with a prefix)
    let response = format_response(message);
    client
        .write_all(response.as_bytes())
        .await
        .map_err(|_| "Failed to send response")?;

    // Close connection
    client
        .close()
        .await
        .map_err(|_| "Failed to close client connection")?;

    Ok(())
}

/// Format a response to send back to the client
fn format_response(client_message: &str) -> String {
    alloc::format!("Server received: {}", client_message)
}
