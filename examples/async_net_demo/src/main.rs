#![no_std]
#![no_main]

extern crate alloc;

use axasync::{block_on, init, shutdown, AsyncRead, AsyncWrite, TcpSocket, TcpSocketExt};
use axlog::info;
use axstd::println;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::panic::PanicInfo;

#[no_mangle]
fn main() {
    // Initialize the async runtime
    init();

    println!("Async Networking Demo");

    // Connect to a TCP echo server (assumes a server is running on this address)
    block_on(async {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        match TcpSocket::connect_to(addr).await {
            Ok(mut socket) => {
                println!("Connected to the server!");

                // Send a message
                let message = b"Hello, async networking!";
                match socket.write_all(message).await {
                    Ok(_) => println!("Sent message: Hello, async networking!"),
                    Err(_) => println!("Failed to send message"),
                }

                // Read the response
                let mut buf = [0u8; 1024];
                match socket.read(&mut buf).await {
                    Ok(n) => {
                        // Convert bytes to string safely
                        let s = core::str::from_utf8(&buf[..n]).unwrap_or("Invalid UTF-8");
                        println!("Received: {}", s);
                    }
                    Err(_) => println!("Failed to read response"),
                }

                // Close the connection
                match socket.close().await {
                    Ok(_) => println!("Connection closed"),
                    Err(_) => println!("Failed to close connection"),
                }
            }
            Err(_) => {
                println!("Failed to connect");
            }
        }
    });

    // Shutdown the async runtime
    shutdown();
}

#[no_mangle]
fn run_tcp_server() {
    block_on(async {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);

        let socket = TcpSocket::new();
        socket.bind(addr).unwrap();
        socket.listen().unwrap();

        println!("Server listening on {}", addr);

        while let Ok(mut client) = socket.accept().await {
            println!("Client connected: {:?}", client.peer_addr().unwrap());

            // Echo back whatever we receive
            let mut buf = [0u8; 1024];
            match client.read(&mut buf).await {
                Ok(n) => {
                    // Convert bytes to string safely
                    let s = core::str::from_utf8(&buf[..n]).unwrap_or("Invalid UTF-8");
                    println!("Received: {}", s);
                    client.write_all(&buf[..n]).await.unwrap();
                }
                Err(_) => {
                    println!("Error reading from client");
                }
            }

            println!("Client disconnected");
        }
    });
}
