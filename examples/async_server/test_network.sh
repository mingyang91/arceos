#!/bin/bash

# Network verification script for StarFive VisionFive 2 running ArceOS
# This script helps verify that the network device is working correctly

BOARD_IP="10.0.2.15"
SERVER_PORT="5555"
TIMEOUT="10"

echo "🔍 ArceOS Network Verification Script"
echo "======================================"
echo "Board IP: $BOARD_IP"
echo "Server Port: $SERVER_PORT"
echo ""

# Function to test network connectivity
test_connectivity() {
    echo "1. Testing basic network connectivity..."
    if ping -c 3 -W $TIMEOUT $BOARD_IP > /dev/null 2>&1; then
        echo "   ✅ Ping successful - Board is reachable"
    else
        echo "   ❌ Ping failed - Board may not be reachable"
        echo "   💡 Check network configuration and board status"
        return 1
    fi
}

# Function to test HTTP server
test_http_server() {
    echo ""
    echo "2. Testing HTTP server..."
    
    # Test with curl
    if command -v curl > /dev/null 2>&1; then
        echo "   Testing with curl..."
        if curl -m $TIMEOUT -s "http://$BOARD_IP:$SERVER_PORT/" > /dev/null 2>&1; then
            echo "   ✅ HTTP server is responding"
            echo "   📄 Server response:"
            curl -m $TIMEOUT -s "http://$BOARD_IP:$SERVER_PORT/" | head -10
        else
            echo "   ❌ HTTP server not responding"
            echo "   💡 Check if the async_server application is running"
        fi
    else
        echo "   ⚠️  curl not available, trying with wget..."
        if command -v wget > /dev/null 2>&1; then
            if wget -T $TIMEOUT -q -O - "http://$BOARD_IP:$SERVER_PORT/" > /dev/null 2>&1; then
                echo "   ✅ HTTP server is responding"
                echo "   📄 Server response:"
                wget -T $TIMEOUT -q -O - "http://$BOARD_IP:$SERVER_PORT/" | head -10
            else
                echo "   ❌ HTTP server not responding"
            fi
        else
            echo "   ⚠️  Neither curl nor wget available"
        fi
    fi
}

# Function to test port connectivity
test_port() {
    echo ""
    echo "3. Testing port connectivity..."
    
    if command -v nc > /dev/null 2>&1; then
        echo "   Testing port $SERVER_PORT with netcat..."
        if echo "GET / HTTP/1.1\r\nHost: $BOARD_IP\r\n\r\n" | nc -w $TIMEOUT $BOARD_IP $SERVER_PORT > /dev/null 2>&1; then
            echo "   ✅ Port $SERVER_PORT is open and responding"
        else
            echo "   ❌ Port $SERVER_PORT is not responding"
        fi
    elif command -v telnet > /dev/null 2>&1; then
        echo "   Testing port $SERVER_PORT with telnet..."
        if timeout $TIMEOUT telnet $BOARD_IP $SERVER_PORT < /dev/null > /dev/null 2>&1; then
            echo "   ✅ Port $SERVER_PORT is open"
        else
            echo "   ❌ Port $SERVER_PORT is not open"
        fi
    else
        echo "   ⚠️  Neither nc nor telnet available for port testing"
    fi
}

# Function to show network information
show_network_info() {
    echo ""
    echo "4. Network Information:"
    echo "   Expected board configuration:"
    echo "   - Interface: eth0"
    echo "   - MAC: 6c-cf-39-00-5d-34"
    echo "   - IP: 10.0.2.15/24"
    echo "   - Gateway: 10.0.2.2"
    echo "   - Server: http://10.0.2.15:5555/"
    echo ""
    echo "   Your network configuration:"
    if command -v ip > /dev/null 2>&1; then
        echo "   Local IP addresses:"
        ip addr show | grep "inet " | grep -v "127.0.0.1" | awk '{print "     " $2}'
    elif command -v ifconfig > /dev/null 2>&1; then
        echo "   Local IP addresses:"
        ifconfig | grep "inet " | grep -v "127.0.0.1" | awk '{print "     " $2}'
    fi
}

# Function to provide troubleshooting tips
show_troubleshooting() {
    echo ""
    echo "🔧 Troubleshooting Tips:"
    echo "======================="
    echo "If the network test fails:"
    echo ""
    echo "1. Check board logs for these messages:"
    echo "   - '=== ASYNC HTTP SERVER STARTING ==='"
    echo "   - 'Application main() function called successfully!'"
    echo "   - '🚀 HTTP Server listening on http://0.0.0.0:5555/'"
    echo ""
    echo "2. Verify network initialization in logs:"
    echo "   - 'NIC 0: \"dwmac-ethernet\", IRQ: 7'"
    echo "   - 'created net interface \"eth0\"'"
    echo "   - 'ether: 6c-cf-39-00-5d-34'"
    echo "   - 'ip: 10.0.2.15/24'"
    echo ""
    echo "3. Common issues:"
    echo "   - Application not reaching main(): Check for panics or hangs"
    echo "   - Network not initialized: Check driver compilation and features"
    echo "   - IP configuration: Verify DHCP or static IP setup"
    echo "   - Firewall: Check if port 5555 is blocked"
    echo ""
    echo "4. Manual testing commands:"
    echo "   ping $BOARD_IP"
    echo "   curl http://$BOARD_IP:$SERVER_PORT/"
    echo "   nc -v $BOARD_IP $SERVER_PORT"
    echo ""
    echo "5. Build command verification:"
    echo "   make A=examples/async_server ARCH=riscv64 PLATFORM=riscv64-starfive \\"
    echo "        LOG=debug NET=y SMP=4 BUS=mmio \\"
    echo "        FEATURES=net,driver-dwmac,bus-mmio \\"
    echo "        APP_FEATURES=default,starfive starfive"
}

# Main execution
main() {
    test_connectivity
    test_http_server
    test_port
    show_network_info
    show_troubleshooting
    
    echo ""
    echo "🏁 Network verification complete!"
    echo "   If all tests pass, your network device is working correctly."
    echo "   If tests fail, check the troubleshooting section above."
}

# Run the main function
main 