#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use axasync::{block_on, init, shutdown, spawn};
use axlog::{debug, error, info};
use axnet::TcpSocket;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};

const LOCAL_PORT: u16 = 5555;

macro_rules! header {
    () => {
        "\
HTTP/1.1 200 OK\r\n\
Content-Type: text/html\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\
\r\n\
{}"
    };
}

const CONTENT: &str = r#"<html>
<head>
  <title>Hello, ArceOS</title>
</head>
<body>
  <center>
    <h1>Hello, <a href="https://github.com/arceos-org/arceos">ArceOS</a></h1>
  </center>
  <hr>
  <center>
    <i>Powered by <a href="https://github.com/arceos-org/arceos/tree/main/examples/httpserver">ArceOS example HTTP server</a> v0.1.0</i>
  </center>
</body>
</html>
"#;

#[no_mangle]
fn main() {
    // Initialize the async runtime
    init();

    info!("Async HTTP Server");

    // Start the HTTP server
    let result = block_on(run_server());
    match result {
        Ok(_) => info!("Server completed successfully"),
        Err(e) => error!("Server error: {}", e),
    }

    // Shutdown the async runtime
    shutdown();
}

/// The main server function that accepts connections and handles client requests
async fn run_server() -> Result<(), &'static str> {
    // Listen on all interfaces on port 5555
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), LOCAL_PORT);

    let socket = TcpSocket::new();
    socket.bind(addr).map_err(|_| "Failed to bind to address")?;
    socket.listen().map_err(|_| "Failed to listen")?;

    info!("HTTP Server listening on http://{}/", addr);
    info!(
        "You can test with a web browser or: curl http://localhost:{}/",
        LOCAL_PORT
    );

    // Keep track of how many connections we've handled
    let mut connection_count = 0;

    // Accept and handle client connections
    loop {
        debug!("Waiting for connection {}...", connection_count + 1);

        match socket.accept_async().await {
            Ok(mut client) => {
                connection_count += 1;
                let connection_count = connection_count;
                spawn(async move {
                    let peer_addr = client
                        .peer_addr()
                        .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0));
                    debug!(
                        "Client connected from {} (connection {})",
                        peer_addr, connection_count
                    );

                    // Handle HTTP request
                    if let Err(e) = handle_http_request(&mut client).await {
                        error!("Error handling HTTP request: {}", e);
                    }

                    debug!("Client disconnected: {}", peer_addr);
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {:?}", e);
            }
        }
    }
}

/// Handle an HTTP request and send an HTML response
async fn handle_http_request(client: &mut TcpSocket) -> Result<(), &'static str> {
    let mut buffer = [0u8; 4096];

    // Read the HTTP request
    let bytes_read = client
        .recv_async(&mut buffer)
        .await
        .map_err(|_| "Failed to read HTTP request")?;

    if bytes_read == 0 {
        // Client closed the connection
        return Ok(());
    }
    debug!("Received {} bytes", bytes_read);

    // Log the request (first line only)
    if let Ok(request_str) = core::str::from_utf8(&buffer[..core::cmp::min(bytes_read, 100)]) {
        if let Some(first_line) = request_str.lines().next() {
            debug!("HTTP Request: {}", first_line);
        }
    }

    let response = format!(header!(), CONTENT.len(), CONTENT);

    // Send the hardcoded HTTP response
    client
        .send_async(response.as_bytes())
        .await
        .map_err(|_| "Failed to send HTTP response")?;

    // Close the connection
    client
        .shutdown()
        .map_err(|_| "Failed to close client connection")?;

    Ok(())
}
