#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use axasync::{block_on, init, shutdown, sleep};
use axnet::TcpSocket;
use axstd::println;
use axstd::time::Duration;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};

// HTTP request to send
const REQUEST: &str = "\
GET / HTTP/1.1\r\n\
Host: ident.me\r\n\
Accept: */*\r\n\
\r\n";

// Maximum number of read operations to prevent infinite loops
const MAX_READ_OPS: usize = 3;

#[no_mangle]
fn main() {
    // Initialize the async runtime
    init();

    println!("Async HTTP Client");

    // Connect to an HTTP server
    let result = block_on(run_http_client());
    match result {
        Ok(_) => println!("HTTP client completed successfully"),
        Err(e) => println!("HTTP client error: {}", e),
    }

    // Shutdown the async runtime
    shutdown();
}

/// The main client function that connects to a server and exchanges HTTP messages
async fn run_http_client() -> Result<(), &'static str> {
    // By default, connect to 10.0.2.2:5555 which is the host machine in QEMU's user networking mode
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(65, 108, 151, 63)), 80);

    println!("Connecting to HTTP server at http://{}...", server_addr);

    let socket = TcpSocket::new();
    socket
        .connect_async(server_addr)
        .await
        .map_err(|_| "Failed to connect to HTTP server")?;

    println!("Connected to HTTP server!");

    // Send HTTP request
    println!("Sending HTTP request: GET / HTTP/1.1");
    socket
        .send_async(REQUEST.as_bytes())
        .await
        .map_err(|_| "Failed to send HTTP request")?;

    // Read the HTTP response
    let mut buffer = [0u8; 4096];
    let mut total_bytes = 0;
    let mut response = String::new();
    let mut read_count = 0;

    println!("Reading HTTP response...");

    loop {
        // Check for maximum read operations to prevent infinite loops
        if read_count >= MAX_READ_OPS {
            println!(
                "Reached maximum read operations ({}), stopping",
                MAX_READ_OPS
            );
            break;
        }

        match socket.recv_async(&mut buffer).await {
            Ok(0) => {
                println!("Server closed the connection");
                break; // EOF
            }
            Ok(n) => {
                total_bytes += n;
                read_count += 1;

                // Convert bytes to string
                if let Ok(chunk) = core::str::from_utf8(&buffer[..n]) {
                    response.push_str(chunk);
                }

                // Small delay to avoid tight loops
                sleep(Duration::from_millis(10)).await;
            }
            Err(_) => return Err("Failed to read HTTP response"),
        }
    }

    // Process and display the response
    println!(
        "Received {} bytes from server in {} read operations",
        total_bytes, read_count
    );

    // Print response headers (first few lines)
    println!("\nHTTP Response Headers:");
    for (i, line) in response.lines().take(10).enumerate() {
        if line.is_empty() {
            println!("<end of headers>");
            break;
        }
        println!("  {}: {}", i + 1, line);
    }

    // Print content length if available
    if let Some(content_length_line) = response
        .lines()
        .find(|line| line.to_lowercase().starts_with("content-length:"))
    {
        println!(
            "Content-Length: {}",
            content_length_line
                .split(":")
                .nth(1)
                .unwrap_or("Unknown")
                .trim()
        );
    }

    // Print a snippet of HTML content
    if let Some(body_start) = response.find("\r\n\r\n") {
        let html_snippet = &response[(body_start + 4)..];
        let snippet_len = core::cmp::min(html_snippet.len(), 100);
        println!("\nHTML Content (first {} bytes):", snippet_len);
        println!("  {}", &html_snippet[..snippet_len]);
        if html_snippet.len() > snippet_len {
            println!("  ... (more content available)");
        }
    }

    // Close the connection
    socket
        .shutdown()
        .map_err(|_| "Failed to close connection")?;

    println!("Connection closed");

    Ok(())
}
