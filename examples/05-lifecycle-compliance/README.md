# Lifecycle Compliance Example

This example demonstrates the MCP 2025-06-18 lifecycle compliance improvements implemented in UltraFast MCP, including proper state transitions and comprehensive timeout configuration.

## Overview

The example showcases two key improvements made to address minor implementation gaps:

1. **Proper State Transitions**: The server now correctly follows the MCP specification by transitioning to `Initialized` state after receiving the `initialize` response, then to `Operating` state only after receiving the `initialized` notification.

2. **Comprehensive Timeout Configuration**: A complete timeout management system that provides configurable timeouts for different operation types, with presets for high-performance and long-running scenarios.

## Features Demonstrated

### State Transition Compliance

- ✅ **Proper Initialization Flow**: Server waits for `initialized` notification before allowing operations
- ✅ **State Validation**: Operations are only allowed in the `Operating` state
- ✅ **Graceful Shutdown**: Proper cleanup and state transitions during shutdown

### Timeout Configuration

- ✅ **Operation-Specific Timeouts**: Different timeouts for different operation types
- ✅ **High-Performance Preset**: Optimized for low-latency scenarios
- ✅ **Long-Running Preset**: Optimized for resource-intensive operations
- ✅ **Custom Configuration**: Full control over all timeout values
- ✅ **Timeout Validation**: Ensures timeouts are within acceptable bounds
- ✅ **Progress Tracking**: Configurable progress notification intervals

## Running the Example

```bash
# From the ultrafast-mcp directory
cargo run --example lifecycle-compliance
```

## Expected Output

The example will demonstrate:

1. **Server Creation**: Shows different timeout configurations
2. **Normal Operation**: A 5-second operation that completes successfully
3. **Timeout Behavior**: A 60-second operation that times out (with high-performance settings)
4. **Configuration Comparison**: Shows different timeout presets
5. **Custom Configuration**: Demonstrates custom timeout settings

## Timeout Configuration Options

### High-Performance Preset
```rust
let server = UltraFastServer::new(info, capabilities)
    .with_high_performance_timeouts();
```
- Tool calls: 30 seconds
- Operations: 60 seconds
- Progress interval: 2 seconds

### Long-Running Preset
```rust
let server = UltraFastServer::new(info, capabilities)
    .with_long_running_timeouts();
```
- Tool calls: 5 minutes
- Operations: 30 minutes
- Progress interval: 10 seconds

### Custom Configuration
```rust
let custom_config = TimeoutConfig::new(
    Duration::from_secs(60),   // initialize_timeout
    Duration::from_secs(600),  // operation_timeout
    Duration::from_secs(120),  // tool_call_timeout
    // ... other timeouts
);

let server = UltraFastServer::new(info, capabilities)
    .with_timeout_config(custom_config);
```

## Implementation Details

### State Transition Fix

**Before (Non-compliant)**:
```rust
// Server transitioned to Operating immediately after initialize
*state = ServerState::Operating;
```

**After (Compliant)**:
```rust
// Server transitions to Initialized, waits for initialized notification
*state = ServerState::Initialized;
```

### Timeout Configuration

The new `TimeoutConfig` struct provides:

- **Granular Control**: Separate timeouts for each operation type
- **Validation**: Ensures timeouts are within acceptable bounds
- **Presets**: Common configurations for different use cases
- **Progress Tracking**: Configurable progress notification intervals

### Operation-Specific Timeouts

```rust
// Get timeout for specific operation
let timeout = server.get_operation_timeout("tools/call");
let timeout = server.get_operation_timeout("resources/read");
let timeout = server.get_operation_timeout("sampling/createMessage");
```

## Compliance Benefits

1. **Protocol Compliance**: Fully compliant with MCP 2025-06-18 lifecycle specification
2. **Better Error Handling**: Proper timeout errors and cancellation notifications
3. **Flexible Configuration**: Adaptable to different deployment scenarios
4. **Production Ready**: Comprehensive timeout management for real-world use
5. **Developer Experience**: Clear APIs and helpful presets

## Testing

The example includes several test scenarios:

- **Normal Operation**: Verifies that operations complete within timeout
- **Timeout Behavior**: Confirms that long operations are properly timed out
- **Configuration Validation**: Ensures timeout configurations are applied correctly
- **State Transitions**: Verifies proper lifecycle state management

This example serves as both a demonstration of the improvements and a template for implementing proper lifecycle management in MCP applications. 