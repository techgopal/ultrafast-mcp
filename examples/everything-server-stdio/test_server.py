#!/usr/bin/env python3
"""
Simple test script for the MCP everything server via STDIO
"""
import json
import subprocess
import sys
import time
import threading

def send_message(process, message):
    """Send a JSON-RPC message to the server"""
    message_str = json.dumps(message) + "\n"
    process.stdin.write(message_str.encode('utf-8'))
    process.stdin.flush()

def read_response(process):
    """Read a JSON-RPC response from the server"""
    try:
        line = process.stdout.readline().decode('utf-8').strip()
        if line:
            return json.loads(line)
    except (json.JSONDecodeError, UnicodeDecodeError) as e:
        print(f"Error reading response: {e}")
        print(f"Raw line: {line}")
    return None

def test_server():
    """Test the MCP server with basic operations"""
    print("🚀 Starting MCP Everything Server test...")
    
    # Start the server process
    server_process = subprocess.Popen(
        ["cargo", "run", "--bin", "server"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=False,
        bufsize=0
    )
    
    try:
        # Test 1: Initialize
        print("\n📋 Test 1: Initialize")
        initialize_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "prompts": {}
                },
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        }
        send_message(server_process, initialize_request)
        response = read_response(server_process)
        if response:
            print("✅ Initialize response:", json.dumps(response, indent=2))
        else:
            print("❌ No initialize response")
            return
        
        # Test 2: List tools
        print("\n🔧 Test 2: List tools")
        tools_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }
        send_message(server_process, tools_request)
        response = read_response(server_process)
        if response:
            print("✅ Tools response:", json.dumps(response, indent=2))
        else:
            print("❌ No tools response")
            return
        
        # Test 3: List resources
        print("\n📁 Test 3: List resources")
        resources_request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "resources/list",
            "params": {}
        }
        send_message(server_process, resources_request)
        response = read_response(server_process)
        if response:
            print("✅ Resources response:", json.dumps(response, indent=2))
        else:
            print("❌ No resources response")
            return
        
        # Test 4: List resource templates
        print("\n📋 Test 4: List resource templates")
        templates_request = {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "resources/templates/list",
            "params": {}
        }
        send_message(server_process, templates_request)
        response = read_response(server_process)
        if response:
            print("✅ Resource templates response:", json.dumps(response, indent=2))
        else:
            print("❌ No resource templates response")
            return
        
        # Test 5: List prompts
        print("\n💬 Test 5: List prompts")
        prompts_request = {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "prompts/list",
            "params": {}
        }
        send_message(server_process, prompts_request)
        response = read_response(server_process)
        if response:
            print("✅ Prompts response:", json.dumps(response, indent=2))
        else:
            print("❌ No prompts response")
            return
        
        # Test 6: Call echo tool
        print("\n🔊 Test 6: Call echo tool")
        echo_request = {
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "echo",
                "arguments": {
                    "message": "Hello from test client!"
                }
            }
        }
        send_message(server_process, echo_request)
        response = read_response(server_process)
        if response:
            print("✅ Echo response:", json.dumps(response, indent=2))
        else:
            print("❌ No echo response")
            return
        
        # Test 7: Shutdown
        print("\n🛑 Test 7: Shutdown")
        shutdown_request = {
            "jsonrpc": "2.0",
            "id": 7,
            "method": "shutdown",
            "params": {}
        }
        send_message(server_process, shutdown_request)
        response = read_response(server_process)
        if response:
            print("✅ Shutdown response:", json.dumps(response, indent=2))
        else:
            print("❌ No shutdown response")
            return
        
        print("\n🎉 All tests completed!")
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
    finally:
        # Clean up
        server_process.terminate()
        server_process.wait()

if __name__ == "__main__":
    test_server() 