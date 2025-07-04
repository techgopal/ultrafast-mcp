# ultrafast-mcp-monitoring

Monitoring and observability for ULTRAFAST MCP with OpenTelemetry support.

This crate provides comprehensive monitoring, metrics, and observability capabilities for MCP servers and clients using OpenTelemetry standards.

## Features

- **OpenTelemetry Integration**: Full OpenTelemetry support for tracing and metrics
- **Multiple Exporters**: Support for Jaeger, Prometheus, and OTLP exporters
- **System Metrics**: CPU, memory, and system resource monitoring
- **Custom Metrics**: Application-specific metrics and counters
- **Health Checks**: Built-in health check endpoints
- **Performance Monitoring**: Request latency and throughput tracking
- **Configuration**: Flexible configuration via TOML and YAML

## Usage

### Basic Setup

```rust
use ultrafast_mcp_monitoring::{Monitoring, MonitoringConfig};

let config = MonitoringConfig::new()
    .with_prometheus(true)
    .with_jaeger("http://localhost:14268")
    .with_system_metrics(true);

let monitoring = Monitoring::new(config);
monitoring.start().await?;
```

### Custom Metrics

```rust
use ultrafast_mcp_monitoring::metrics::{counter, gauge, histogram};

// Increment a counter
counter!("requests_total", 1, "endpoint" => "/api/v1/resources");

// Set a gauge
gauge!("active_connections", 42.0);

// Record a histogram
histogram!("request_duration", 0.15, "endpoint" => "/api/v1/resources");
```

## Features

- `prometheus` - Enables Prometheus metrics exporter (default)
- `jaeger` - Enables Jaeger tracing exporter (default)
- `otlp` - Enables OTLP exporter
- `system-metrics` - Enables system resource monitoring
- `console` - Enables console output for development (default)
- `config-files` - Enables TOML/YAML configuration support
- `all` - Enables all features

## Dependencies

- `opentelemetry` - OpenTelemetry core
- `tracing` - Logging and tracing
- `metrics` - Metrics collection
- `sysinfo` - System information

## License

MIT OR Apache-2.0 