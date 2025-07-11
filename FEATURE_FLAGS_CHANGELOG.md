# Feature Flags Standardization Changelog

## Overview
This document tracks the standardization of feature flags across all crates in the UltraFast MCP project to ensure consistency, predictability, and better developer experience.

## Changes Made

### 1. **Standardized Default Features**
- **Before**: Some crates had default features (`ultrafast-mcp-transport`, `ultrafast-mcp-auth`, `ultrafast-mcp-monitoring`)
- **After**: All crates now have `default = []` for minimal footprint
- **Impact**: Users must explicitly choose features, preventing unexpected dependencies

### 2. **Removed Empty Features**
- **Before**: `oauth = []` in `ultrafast-mcp-server` and `ultrafast-mcp-auth`
- **After**: Removed empty feature definitions
- **Impact**: Cleaner feature definitions without confusing empty features

### 3. **Standardized Feature Naming**
- **Before**: Inconsistent naming (`core`, `full`, `all`, `minimal`)
- **After**: Consistent naming across all crates:
  - `core` - Basic functionality
  - `full` - All features for the crate
  - `minimal` - Minimal working setup (main crate only)

### 4. **Fixed Dependency Propagation**
- **Before**: `http` feature in `ultrafast-mcp-client` auto-included auth
- **After**: `http` and `oauth` are separate, with `http-with-auth` convenience feature
- **Impact**: Clear separation of concerns and predictable dependencies

### 5. **Added Granular Monitoring Features**
- **Before**: Only `monitoring` feature
- **After**: Granular monitoring features:
  - `monitoring-http` - HTTP metrics endpoints
  - `monitoring-jaeger` - Jaeger tracing
  - `monitoring-otlp` - OTLP tracing
  - `monitoring-console` - Console output
  - `monitoring-full` - All monitoring features

### 6. **Added Convenience Combinations**
- `http-with-auth` - HTTP transport + OAuth authentication
- `monitoring-full` - All monitoring features
- `minimal` - Core + STDIO (minimal working setup)

## Updated Crates

### **ultrafast-mcp** (Main Crate)
```toml
[features]
default = []
core = []
stdio = ["ultrafast-mcp-transport/stdio"]
http = ["ultrafast-mcp-transport/http", "ultrafast-mcp-server/http", "ultrafast-mcp-client/http"]
oauth = ["ultrafast-mcp-auth/oauth"]
monitoring = ["ultrafast-mcp-monitoring"]
monitoring-http = ["ultrafast-mcp-monitoring/http"]
monitoring-jaeger = ["ultrafast-mcp-monitoring/jaeger"]
monitoring-otlp = ["ultrafast-mcp-monitoring/otlp"]
monitoring-console = ["ultrafast-mcp-monitoring/console"]
http-with-auth = ["http", "oauth"]
monitoring-full = ["ultrafast-mcp-monitoring/all"]
minimal = ["core", "stdio"]
full = ["core", "stdio", "http", "oauth", "monitoring-full"]
```

### **ultrafast-mcp-core**
```toml
[features]
default = []
core = []
full = ["core"]
```

### **ultrafast-mcp-transport**
```toml
[features]
default = []
core = []
stdio = []
http = ["axum", "reqwest", "tower-http", "axum-extra", "bytes"]
full = ["core", "stdio", "http"]
```

### **ultrafast-mcp-server**
```toml
[features]
default = []
core = []
monitoring = ["ultrafast-mcp-monitoring"]
http = ["ultrafast-mcp-transport/http"]
full = ["core", "monitoring", "http"]
```

### **ultrafast-mcp-client**
```toml
[features]
default = []
core = []
http = ["ultrafast-mcp-transport/http"]
oauth = ["ultrafast-mcp-auth/oauth"]
http-with-auth = ["http", "oauth"]
full = ["core", "http", "oauth"]
```

### **ultrafast-mcp-auth**
```toml
[features]
default = []
core = []
oauth = []
full = ["core", "oauth"]
```

### **ultrafast-mcp-monitoring**
```toml
[features]
default = []
core = []
http = ["dep:axum", "dep:tower", "dep:opentelemetry-prometheus", "dep:metrics", "dep:metrics-exporter-prometheus"]
jaeger = ["dep:opentelemetry-jaeger", "dep:tracing-opentelemetry"]
otlp = ["dep:opentelemetry-otlp", "dep:tracing-opentelemetry"]
console = ["dep:tracing-opentelemetry"]
config-files = ["dep:toml", "dep:serde_yaml"]
all = ["core", "http", "jaeger", "otlp", "console", "config-files"]
```

### **ultrafast-mcp-cli**
```toml
[features]
default = []
core = []
auth = ["ultrafast-mcp-auth"]
monitoring = ["ultrafast-mcp-monitoring"]
full = ["core", "auth", "monitoring"]
```

### **ultrafast-mcp-test-utils**
```toml
[features]
default = []
core = []
full = ["core"]
```

## Updated Examples

### **01-basic-echo**
- **Before**: `features = ["http", "oauth"]`
- **After**: `features = ["http-with-auth"]`

### **02-file-operations**
- **Before**: `features = ["http"]`
- **After**: `features = ["http"]` (unchanged)

### **03-everything-server**
- **Before**: `features = ["http", "monitoring", "oauth"]`
- **After**: `features = ["http-with-auth", "monitoring-full"]`

### **04-authentication-example**
- **Before**: `features = ["oauth"]`
- **After**: `features = ["oauth"]` (unchanged)

## Migration Guide

### For Users

#### **Minimal Setup**
```bash
# Before: No explicit features needed
cargo add ultrafast-mcp

# After: Explicit minimal features
cargo add ultrafast-mcp --features="minimal"
```

#### **HTTP Server**
```bash
# Before: HTTP + OAuth auto-included
cargo add ultrafast-mcp --features="http"

# After: HTTP only
cargo add ultrafast-mcp --features="http"

# After: HTTP + OAuth (explicit)
cargo add ultrafast-mcp --features="http-with-auth"
```

#### **Production Setup**
```bash
# Before: Basic monitoring
cargo add ultrafast-mcp --features="http,oauth,monitoring"

# After: Full monitoring suite
cargo add ultrafast-mcp --features="http-with-auth,monitoring-full"
```

### For Developers

#### **Adding New Features**
1. Use consistent naming (`core`, `full`, `minimal`)
2. No default features unless absolutely necessary
3. Provide granular features for complex functionality
4. Add convenience combinations for common use cases

#### **Feature Dependencies**
1. Keep features independent when possible
2. Use convenience features for common combinations
3. Document feature dependencies clearly
4. Test all feature combinations

## Benefits

1. **Predictable Behavior**: No unexpected dependencies from default features
2. **Minimal Footprint**: Users can choose exactly what they need
3. **Clear Documentation**: Consistent naming makes features easier to understand
4. **Better Testing**: Granular features enable better test coverage
5. **Flexible Deployment**: Users can optimize for their specific use case

## Breaking Changes

⚠️ **This is a breaking change for users relying on default features:**

- Users of `ultrafast-mcp-transport` must now explicitly enable `stdio`
- Users of `ultrafast-mcp-auth` must now explicitly enable `oauth`
- Users of `ultrafast-mcp-monitoring` must now explicitly enable desired features

**Migration**: Add appropriate features to your `Cargo.toml`:
```toml
ultrafast-mcp = { version = "20250618.1.0-rc.2", features = ["minimal"] }
```

## Future Considerations

1. **Version Bumping**: Consider bumping to RC.3 for this breaking change
2. **Documentation**: Update all documentation to reflect new feature flags
3. **Examples**: Ensure all examples use appropriate feature combinations
4. **Testing**: Add comprehensive tests for all feature combinations 