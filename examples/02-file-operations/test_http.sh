#!/bin/bash

# File Operations MCP Example - HTTP Transport Test
# This script tests the file operations server and client using HTTP transport

set -e

echo "🚀 UltraFast MCP File Operations - HTTP Transport Test"
echo "======================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SERVER_HOST="127.0.0.1"
SERVER_PORT="8080"
SERVER_URL="http://${SERVER_HOST}:${SERVER_PORT}"
TEST_DIR="/tmp/mcp_file_ops_test"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    print_status "Cleaning up..."
    if [ -d "$TEST_DIR" ]; then
        rm -rf "$TEST_DIR"
        print_success "Removed test directory: $TEST_DIR"
    fi
    
    # Kill background processes
    if [ ! -z "$SERVER_PID" ]; then
        print_status "Stopping server (PID: $SERVER_PID)"
        kill $SERVER_PID 2>/dev/null || true
    fi
    
    print_success "Cleanup completed"
}

# Set up cleanup on script exit
trap cleanup EXIT

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    print_error "cargo is not installed or not in PATH"
    exit 1
fi

# Build the project
print_status "Building project..."
cargo build --release
print_success "Build completed"

# Create test directory
print_status "Creating test directory: $TEST_DIR"
mkdir -p "$TEST_DIR"
print_success "Test directory created"

# Start the server in background
print_status "Starting server on $SERVER_HOST:$SERVER_PORT"
./target/release/file-ops-server http --host "$SERVER_HOST" --port "$SERVER_PORT" &
SERVER_PID=$!

# Wait for server to start
print_status "Waiting for server to start..."
sleep 3

# Check if server is running
if ! curl -s "$SERVER_URL" > /dev/null 2>&1; then
    print_error "Server is not responding on $SERVER_URL"
    exit 1
fi
print_success "Server is running and responding"

# Test the client
print_status "Testing client connection..."
./target/release/file-ops-client http --server-url "$SERVER_URL"

print_success "All tests completed successfully!"
echo ""
echo "🎉 HTTP transport test passed!"
echo "   Server: $SERVER_URL"
echo "   Test directory: $TEST_DIR" 