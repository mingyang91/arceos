# Async HTTP Server Example for ArceOS

This example demonstrates an asynchronous HTTP server running on ArceOS, specifically tested on the StarFive VisionFive 2 RISC-V board.

## Features

- Asynchronous HTTP server using ArceOS async runtime
- Network device support via DWMAC Ethernet driver
- Multi-core support (SMP)
- Comprehensive logging and diagnostics

## Build Instructions

### For StarFive VisionFive 2

```bash
make A=examples/async_server ARCH=riscv64 PLATFORM=riscv64-starfive \
     LOG=debug NET=y SMP=4 BUS=mmio \
     FEATURES=net,driver-dwmac,bus-mmio \
     APP_FEATURES=default,starfive starfive
```

### Build Parameters Explained

- `A=examples/async_server`: Target application
- `ARCH=riscv64`: RISC-V 64-bit architecture
- `PLATFORM=riscv64-starfive`: StarFive VisionFive 2 platform
- `LOG=debug`: Enable debug logging
- `NET=y`: Enable network support
- `SMP=4`: Enable 4-core SMP support
- `BUS=mmio`: Memory-mapped I/O bus support
- `FEATURES=net,driver-dwmac,bus-mmio`: Core features
- `APP_FEATURES=default,starfive`: Application-specific features

## Expected Boot Sequence

When the system boots correctly, you should see these log messages in order:

### 1. Network Device Initialization
```
[ 12.391853 axdriver:166]   NIC 0: "dwmac-ethernet", IRQ: 7
[ 12.398539 axnet:42] Initialize network subsystem...
[ 12.404689 axnet:45]   use NIC 0: "dwmac-ethernet", IRQ: 7
```

### 2. Network Interface Creation
```
[ 12.453674 axnet::smoltcp_impl:338] created net interface "eth0":
[ 12.460954 axnet::smoltcp_impl:339]   ether:    6c-cf-39-00-5d-34
[ 12.468234 axnet::smoltcp_impl:340]   ip:       10.0.2.15/24
[ 12.475080 axnet::smoltcp_impl:341]   gateway:  10.0.2.2
[ 12.481580 axnet::smoltcp_impl:342]   IRQ:      7
```

### 3. CPU Initialization
```
[ 12.487474 axruntime::mp:17] VisionFive 2: Starting secondary CPUs 1-3
[ 12.537310 axruntime:191] Primary CPU 1 init OK.
```

### 4. Application Start (Expected)
```
[timestamp] === ASYNC HTTP SERVER STARTING ===
[timestamp] Application main() function called successfully!
[timestamp] Checking network subsystem...
[timestamp] Initializing async runtime...
[timestamp] Async runtime initialized successfully
[timestamp] Testing socket creation...
[timestamp] ✅ Socket creation successful
[timestamp] Starting HTTP server...
[timestamp] === Starting HTTP Server ===
[timestamp] Attempting to bind to address: 0.0.0.0:5555
[timestamp] Socket created successfully
[timestamp] Socket bound successfully to 0.0.0.0:5555
[timestamp] 🚀 HTTP Server listening on http://0.0.0.0:5555/
[timestamp] 📡 Network interface: eth0 (6c-cf-39-00-5d-34)
[timestamp] 🌐 IP: 10.0.2.15/24, Gateway: 10.0.2.2
[timestamp] 🔧 Test with: curl http://10.0.2.15:5555/
[timestamp] 💡 Or browse to: http://10.0.2.15:5555/
[timestamp] Waiting for connection 1...
```

## Network Verification

### Automatic Testing

Use the provided test script to verify network functionality:

```bash
./examples/async_server/test_network.sh
```

This script will:
1. Test basic network connectivity (ping)
2. Test HTTP server response
3. Test port connectivity
4. Show network configuration
5. Provide troubleshooting tips

### Manual Testing

#### 1. Basic Connectivity
```bash
ping 10.0.2.15
```

#### 2. HTTP Server Test
```bash
curl http://10.0.2.15:5555/
```

#### 3. Port Test
```bash
nc -v 10.0.2.15 5555
```

#### 4. Browser Test
Open `http://10.0.2.15:5555/` in your web browser.

## Troubleshooting

### Issue: Application main() not called

**Symptoms:**
- Network device initializes correctly
- CPU initialization completes
- No "=== ASYNC HTTP SERVER STARTING ===" message

**Possible Causes:**
1. **Panic during runtime initialization**
   - Check for panic messages in logs
   - Look for memory allocation failures
   - Verify all required features are enabled

2. **Infinite loop in initialization**
   - Check if system hangs after "Primary CPU init OK"
   - May indicate deadlock in async runtime setup

3. **Missing dependencies**
   - Verify all required crates are linked
   - Check for missing symbols during linking

**Debug Steps:**
1. Add early debug prints in `rust_main()` function
2. Check memory allocation logs
3. Verify async runtime dependencies
4. Test with simpler applications first

### Issue: Network device not working

**Symptoms:**
- No network interface creation logs
- Missing "created net interface eth0" message

**Solutions:**
1. Verify build features include `net,driver-dwmac,bus-mmio`
2. Check device tree configuration
3. Verify DWMAC driver compilation
4. Check IRQ configuration

### Issue: HTTP server not responding

**Symptoms:**
- Application starts correctly
- Server binding succeeds
- No response to HTTP requests

**Solutions:**
1. Check firewall settings
2. Verify IP address configuration
3. Test with different ports
4. Check network routing

## Network Configuration

### Default Configuration
- **Interface:** eth0
- **MAC Address:** 6c-cf-39-00-5d-34
- **IP Address:** 10.0.2.15/24
- **Gateway:** 10.0.2.2
- **Server Port:** 5555

### Customization

To change the server port, modify `LOCAL_PORT` in `src/main.rs`:

```rust
const LOCAL_PORT: u16 = 8080;  // Change to desired port
```

To change IP configuration, modify the network setup in the platform configuration.

## Performance Notes

- The server handles connections asynchronously
- Each connection is processed in a separate async task
- Memory usage scales with concurrent connections
- Tested with up to 100 concurrent connections

## Development Tips

1. **Enable debug logging** for detailed diagnostics
2. **Use the test script** for quick verification
3. **Monitor memory usage** during development
4. **Test incrementally** - start with simple examples
5. **Check logs carefully** - timing issues are common

## Related Examples

- `examples/httpserver` - Synchronous HTTP server
- `examples/net` - Basic networking examples
- `examples/task` - Task and async examples

## Support

For issues specific to:
- **StarFive VisionFive 2**: Check hardware documentation
- **DWMAC driver**: Review driver implementation in `modules/axdriver`
- **Network stack**: Check `modules/axnet` implementation
- **Async runtime**: Review `modules/axasync` documentation 