//! # UltraFast MCP Monitoring
//!
//! Comprehensive monitoring and observability system for the Model Context Protocol (MCP).
//!
//! This crate provides a complete monitoring solution for MCP servers and clients,
//! including metrics collection, health checking, distributed tracing, and
//! OpenTelemetry integration. It's designed to provide deep insights into MCP
//! application performance and health.
//!
//! ## Overview
//!
//! The UltraFast MCP Monitoring system provides:
//!
//! - **Metrics Collection**: Comprehensive request, transport, and system metrics
//! - **Health Checking**: Application and system health monitoring
//! - **Distributed Tracing**: End-to-end request tracing with OpenTelemetry
//! - **Performance Monitoring**: Response times, throughput, and resource usage
//! - **Alerting**: Configurable alerts for performance and health issues
//! - **Exporters**: Prometheus, JSON, and custom metric exporters
//!
//! ## Key Features
//!
//! ### Metrics Collection
//! - **Request Metrics**: Count, timing, and success rate tracking
//! - **Transport Metrics**: Network I/O, connection counts, and errors
//! - **System Metrics**: Memory, CPU, and resource usage monitoring
//! - **Custom Metrics**: Extensible metric system for application-specific data
//! - **Real-time Updates**: Live metric updates with minimal overhead
//!
//! ### Health Checking
//! - **System Health**: CPU, memory, and resource availability checks
//! - **Application Health**: Service availability and dependency checks
//! - **Custom Health Checks**: Application-specific health validation
//! - **Health Aggregation**: Combined health status reporting
//! - **Health History**: Historical health data and trends
//!
//! ### Distributed Tracing
//! - **Request Tracing**: End-to-end request flow tracking
//! - **Span Management**: Automatic span creation and management
//! - **Context Propagation**: Trace context across service boundaries
//! - **OpenTelemetry Integration**: Standard tracing protocol support
//! - **Trace Export**: Export traces to various backends
//!
//! ### Performance Monitoring
//! - **Response Time Tracking**: Detailed timing analysis
//! - **Throughput Monitoring**: Request rate and capacity planning
//! - **Resource Usage**: Memory, CPU, and network utilization
//! - **Error Rate Tracking**: Failure rate and error categorization
//! - **Performance Alerts**: Configurable performance thresholds
//!
//! ## Modules
//!
//! - **[`config`]**: Monitoring configuration and settings
//! - **[`metrics`]**: Metrics collection and management
//! - **[`health`]**: Health checking and status monitoring
//! - **[`tracing`]**: Distributed tracing and OpenTelemetry integration
//! - **[`exporters`]**: Metric and trace exporters
//! - **[`middleware`]**: Monitoring middleware for HTTP and transport layers
//!
//! ## Usage Examples
//!
//! ### Basic Monitoring Setup
//!
//! ```rust
//! use ultrafast_mcp_monitoring::{
//!     MonitoringSystem, MonitoringConfig, RequestTimer
//! };
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create monitoring configuration
//!     let config = MonitoringConfig::default();
//!
//!     // Initialize monitoring system
//!     let monitoring = MonitoringSystem::init(config).await?;
//!
//!     // Use monitoring in your application
//!     let metrics = monitoring.metrics();
//!     let timer = RequestTimer::start("tools/call", metrics.clone());
//!
//!     // ... perform your operation ...
//!
//!     // Record the request completion
//!     timer.finish(true).await;
//!
//!     // Start HTTP monitoring server (requires http feature)
//!     #[cfg(feature = "http")]
//!     {
//!         let addr = "127.0.0.1:9090".parse()?;
//!         monitoring.start_http_server(addr).await?;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Custom Health Checks
//!
//! ```rust
//! use ultrafast_mcp_monitoring::{
//!     MonitoringSystem, HealthChecker, HealthStatus
//! };
//! use ultrafast_mcp_monitoring::health::{HealthCheck, HealthCheckResult};
//! use std::time::{Duration, SystemTime};
//!
//! struct DatabaseHealthCheck;
//!
//! #[async_trait::async_trait]
//! impl HealthCheck for DatabaseHealthCheck {
//!     async fn check(&self) -> HealthCheckResult {
//!         let start = std::time::Instant::now();
//!         
//!         // Implement your database health check
//!         let status = match check_database_connection().await {
//!             Ok(_) => HealthStatus::Healthy,
//!             Err(e) => HealthStatus::Unhealthy(format!("Database error: {}", e)),
//!         };
//!
//!         HealthCheckResult {
//!             status,
//!             duration: start.elapsed(),
//!             timestamp: SystemTime::now(),
//!         }
//!     }
//!
//!     fn name(&self) -> &str {
//!         "database"
//!     }
//! }
//!
//! async fn check_database_connection() -> anyhow::Result<()> {
//!     // Implement database connection check
//!     Ok(())
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let monitoring = MonitoringSystem::init(MonitoringConfig::default()).await?;
//!     let health_checker = monitoring.health();
//!
//!     // Add custom health check
//!     health_checker.add_check(Box::new(DatabaseHealthCheck)).await;
//!
//!     // Check overall health
//!     match health_checker.check_all().await {
//!         HealthStatus::Healthy => println!("All systems healthy"),
//!         HealthStatus::Degraded(warnings) => {
//!             println!("System degraded: {:?}", warnings);
//!         }
//!         HealthStatus::Unhealthy(errors) => {
//!             println!("System unhealthy: {:?}", errors);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Custom Metrics
//!
//! ```rust
//! use ultrafast_mcp_monitoring::{MonitoringSystem, MetricsCollector};
//! use std::collections::HashMap;
//!
//! async fn record_custom_metrics(monitoring: &MonitoringSystem) {
//!     let metrics = monitoring.metrics();
//!
//!     // Record transport metrics
//!     metrics.record_transport_send(1024).await;
//!     metrics.record_transport_receive(2048).await;
//!
//!     // Update system metrics
//!     metrics.update_system_metrics(10, 1024 * 1024, 25.5).await;
//!
//!     // Get current metrics
//!     let current_metrics = metrics.get_metrics().await;
//!     println!("Total requests: {}", current_metrics.request.total_requests);
//!     println!("Memory usage: {} bytes", current_metrics.system.memory_usage);
//! }
//! ```
//!
//! ### Distributed Tracing
//!
//! ```rust
//! use ultrafast_mcp_monitoring::config::TracingConfig;
//! use tracing::{info, error};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize tracing configuration
//!     let mut tracing_config = TracingConfig::default();
//!     tracing_config.service_name = "my-mcp-server".to_string();
//!     tracing_config.service_version = "1.0.0".to_string();
//!
//!     // Use tracing in your application
//!     info!("Starting MCP server");
//!
//!     // Create spans for operations
//!     let span = tracing::info_span!("process_request", method = "tools/call");
//!     let _enter = span.enter();
//!
//!     // ... perform operation ...
//!
//!     info!("Request processed successfully");
//!
//!     Ok(())
//! }
//! ```
//!
//! ### HTTP Middleware Integration
//!
//! ```rust,no_run
//! use ultrafast_mcp_monitoring::MonitoringSystem;
//! #[cfg(feature = "http")]
//! use axum::{Router, routing::post};
//!
//! #[cfg(feature = "http")]
//! async fn create_app(monitoring: MonitoringSystem) -> Router {
//!     Router::new()
//!         .route("/mcp", post(handle_mcp_request))
//! }
//!
//! async fn handle_mcp_request() -> &'static str {
//!     "OK"
//! }
//! ```
//!
//! ## Configuration
//!
//! ### Basic Configuration
//! ```rust
//! use ultrafast_mcp_monitoring::MonitoringConfig;
//!
//! let config = MonitoringConfig::default();
//! ```
//!
//! ### Advanced Configuration
//! ```rust
//! use ultrafast_mcp_monitoring::MonitoringConfig;
//! use std::time::Duration;
//!
//! let mut config = MonitoringConfig::default();
//! config.metrics.enabled = true;
//! config.health.enabled = true;
//! config.tracing.enabled = true;
//! config.http.enabled = true;
//! config.http.address = "127.0.0.1".to_string();
//! config.http.port = 9090;
//! config.metrics.collection_interval = Duration::from_secs(30);
//! ```
//!
//! ## Metrics Types
//!
//! ### Request Metrics
//! - **Total Requests**: Count of all requests processed
//! - **Successful Requests**: Count of successful requests
//! - **Failed Requests**: Count of failed requests
//! - **Average Response Time**: Mean response time across all requests
//! - **Method Counts**: Request count by MCP method
//!
//! ### Transport Metrics
//! - **Bytes Sent**: Total bytes sent over the network
//! - **Bytes Received**: Total bytes received from the network
//! - **Connection Count**: Number of active connections
//! - **Error Count**: Number of transport errors
//!
//! ### System Metrics
//! - **Memory Usage**: Current memory consumption in bytes
//! - **CPU Usage**: Current CPU utilization percentage
//! - **Active Connections**: Number of active network connections
//! - **Uptime**: Application uptime duration
//!
//! ## Health Check Types
//!
//! ### System Health Checks
//! - **Memory Check**: Verify available memory
//! - **CPU Check**: Monitor CPU utilization
//! - **Disk Check**: Verify disk space availability
//! - **Network Check**: Test network connectivity
//!
//! ### Application Health Checks
//! - **Service Check**: Verify service availability
//! - **Database Check**: Test database connectivity
//! - **Dependency Check**: Verify external service dependencies
//! - **Custom Checks**: Application-specific health validation
//!
//! ## Exporters
//!
//! ### Prometheus Exporter
//! Exports metrics in Prometheus format for integration with monitoring systems.
//!
//! ```bash
//! # Access metrics endpoint
//! curl http://localhost:9090/metrics
//!
//! # Example Prometheus configuration
//! scrape_configs:
//!   - job_name: 'mcp-server'
//!     static_configs:
//!       - targets: ['localhost:9090']
//!     metrics_path: '/metrics'
//! ```
//!
//! ### JSON Exporter
//! Exports metrics in JSON format for custom integrations.
//!
//! ```bash
//! # Access JSON metrics
//! curl http://localhost:9090/metrics/json
//! ```
//!
//! ### Jaeger Exporter
//! Exports traces to Jaeger for distributed tracing visualization.
//!
//! ```rust
//! use ultrafast_mcp_monitoring::config::TracingConfig;
//! 
//! let mut config = TracingConfig::default();
//! config.jaeger = Some(ultrafast_mcp_monitoring::config::JaegerConfig {
//!     agent_endpoint: "http://localhost:14268/api/traces".to_string(),
//!     collector_endpoint: None,
//!     headers: std::collections::HashMap::new(),
//! });
//! ```
//!
//! ## Performance Considerations
//!
//! - **Minimal Overhead**: Optimized for minimal performance impact
//! - **Async Operations**: All monitoring operations are asynchronous
//! - **Efficient Storage**: Optimized metric storage and retrieval
//! - **Batch Processing**: Batch metric updates for efficiency
//! - **Memory Management**: Efficient memory usage and cleanup
//!
//! ## Thread Safety
//!
//! All monitoring components are designed to be thread-safe:
//! - Metrics collectors are `Send + Sync`
//! - Health checkers support concurrent access
//! - Tracing systems are thread-safe
//! - No mutable global state is used
//!
//! ## Best Practices
//!
//! ### Monitoring Setup
//! - Enable monitoring early in development
//! - Use appropriate metric retention periods
//! - Configure meaningful health checks
//! - Set up alerting for critical issues
//! - Monitor both application and system metrics
//!
//! ### Performance Monitoring
//! - Track response times for all operations
//! - Monitor error rates and failure patterns
//! - Track resource usage and capacity
//! - Set up performance baselines
//! - Use percentiles for response time analysis
//!
//! ### Health Checking
//! - Implement comprehensive health checks
//! - Use appropriate timeouts for health checks
//! - Monitor external dependencies
//! - Implement graceful degradation
//! - Provide detailed health status information
//!
//! ### Tracing
//! - Use meaningful span names and attributes
//! - Propagate trace context across services
//! - Implement proper error handling in spans
//! - Use sampling for high-traffic applications
//! - Monitor trace performance impact
//!
//! ## Examples
//!
//! See the `examples/` directory for complete working examples:
//! - Basic monitoring setup
//! - Custom health checks
//! - Distributed tracing
//! - Metric exporters
//! - HTTP middleware integration

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[cfg(feature = "http")]
use axum::{routing::get, Router};

pub mod config;
pub mod exporters;
pub mod health;
pub mod metrics;
pub mod middleware;
pub mod tracing;

pub use config::MonitoringConfig;
pub use health::{HealthChecker, HealthStatus};

/// Main metrics structure for the monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub request: RequestMetrics,
    pub transport: TransportMetrics,
    pub system: SystemMetrics,
}

/// Request-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub method_counts: HashMap<String, u64>,
}

/// Transport-level metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransportMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_count: u32,
    pub error_count: u64,
}

/// System-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub active_connections: u32,
    pub uptime: Duration,
}

/// The main monitoring system that orchestrates all monitoring components
#[derive(Clone)]
pub struct MonitoringSystem {
    pub metrics_collector: Arc<MetricsCollector>,
    pub health_checker: Arc<HealthChecker>,
    pub config: MonitoringConfig,
}

impl MonitoringSystem {
    /// Create a new monitoring system with configuration (synchronous)
    pub fn new(config: MonitoringConfig) -> Self {
        let health_checker = Arc::new(HealthChecker::new());
        let metrics_collector = Arc::new(MetricsCollector::new());

        Self {
            metrics_collector,
            health_checker,
            config,
        }
    }

    /// Initialize the monitoring system with configuration (async version with health checks)
    pub async fn init(config: MonitoringConfig) -> Result<Self> {
        let health_checker = Arc::new(HealthChecker::new());

        // Add basic health checks
        health_checker
            .add_check(Box::new(health::SystemHealthCheck::new()))
            .await;

        let metrics_collector = Arc::new(MetricsCollector::new());

        Ok(Self {
            metrics_collector,
            health_checker,
            config,
        })
    }

    /// Get a reference to the metrics collector
    pub fn metrics(&self) -> Arc<MetricsCollector> {
        self.metrics_collector.clone()
    }

    /// Get a reference to the health checker
    pub fn health(&self) -> Arc<HealthChecker> {
        self.health_checker.clone()
    }

    /// Initialize health checks asynchronously
    pub async fn init_health_checks(&self) -> Result<()> {
        self.health_checker
            .add_check(Box::new(health::SystemHealthCheck::new()))
            .await;
        Ok(())
    }

    /// Start the HTTP monitoring server
    #[cfg(feature = "http")]
    pub async fn start_http_server(&self, addr: std::net::SocketAddr) -> Result<()> {
        let app = Router::new()
            .route(
                "/metrics",
                get({
                    let collector = self.metrics_collector.clone();
                    move || async move { collector.export_prometheus().await }
                }),
            )
            .route(
                "/health",
                get({
                    let health = self.health_checker.clone();
                    move || async move {
                        match health.check_all().await {
                            HealthStatus::Healthy => "OK",
                            HealthStatus::Unhealthy(_) => "UNHEALTHY",
                            HealthStatus::Degraded(_) => "DEGRADED",
                        }
                    }
                }),
            )
            .route(
                "/metrics/json",
                get({
                    let collector = self.metrics_collector.clone();
                    move || async move {
                        match collector.export_json().await {
                            Ok(json) => json,
                            Err(_) => "{}".to_string(),
                        }
                    }
                }),
            );

        println!("Starting monitoring HTTP server on {}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }

    /// Start the HTTP monitoring server (stub when http feature is disabled)
    #[cfg(not(feature = "http"))]
    pub async fn start_http_server(&self, _addr: std::net::SocketAddr) -> Result<()> {
        Err(anyhow::anyhow!("HTTP server requires 'http' feature to be enabled"))
    }

    /// Shutdown the monitoring system
    pub async fn shutdown(&self) -> Result<()> {
        println!("Shutting down monitoring system");
        Ok(())
    }
}

/// Metrics collector that handles all metric recording and export
pub struct MetricsCollector {
    pub metrics: Arc<tokio::sync::RwLock<Metrics>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(tokio::sync::RwLock::new(Metrics {
                request: RequestMetrics::default(),
                transport: TransportMetrics::default(),
                system: SystemMetrics::default(),
            })),
        }
    }

    /// Record a request with timing and success information
    pub async fn record_request(&self, method: &str, response_time: Duration, success: bool) {
        let mut metrics = self.metrics.write().await;

        metrics.request.total_requests += 1;
        if success {
            metrics.request.successful_requests += 1;
        } else {
            metrics.request.failed_requests += 1;
        }

        // Update method count
        *metrics
            .request
            .method_counts
            .entry(method.to_string())
            .or_insert(0) += 1;

        // Update average response time (simple moving average)
        let current_avg = metrics.request.average_response_time;
        let total_requests = metrics.request.total_requests as f64;
        let new_time = response_time.as_secs_f64();

        if total_requests == 1.0 {
            metrics.request.average_response_time = new_time;
        } else {
            metrics.request.average_response_time =
                (current_avg * (total_requests - 1.0) + new_time) / total_requests;
        }
    }

    /// Record transport send operation
    pub async fn record_transport_send(&self, bytes: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.transport.bytes_sent += bytes;
    }

    /// Record transport receive operation
    pub async fn record_transport_receive(&self, bytes: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.transport.bytes_received += bytes;
    }

    /// Record transport error
    pub async fn record_transport_error(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.transport.error_count += 1;
    }

    /// Update system metrics
    pub async fn update_system_metrics(
        &self,
        active_connections: u32,
        memory_usage: u64,
        cpu_usage: f64,
    ) {
        let mut metrics = self.metrics.write().await;
        metrics.system.active_connections = active_connections;
        metrics.system.memory_usage = memory_usage;
        metrics.system.cpu_usage = cpu_usage;
    }

    /// Get current metrics snapshot
    pub async fn get_metrics(&self) -> Metrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Export metrics as JSON
    pub async fn export_json(&self) -> serde_json::Result<String> {
        let metrics = self.get_metrics().await;
        serde_json::to_string_pretty(&metrics)
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let metrics = self.get_metrics().await;

        let mut output = String::new();

        // Request metrics
        output.push_str("# HELP mcp_requests_total Total number of requests\n");
        output.push_str("# TYPE mcp_requests_total counter\n");
        output.push_str(&format!(
            "mcp_requests_total {}\n",
            metrics.request.total_requests
        ));

        output.push_str("# HELP mcp_requests_successful Total number of successful requests\n");
        output.push_str("# TYPE mcp_requests_successful counter\n");
        output.push_str(&format!(
            "mcp_requests_successful {}\n",
            metrics.request.successful_requests
        ));

        output.push_str("# HELP mcp_requests_failed Total number of failed requests\n");
        output.push_str("# TYPE mcp_requests_failed counter\n");
        output.push_str(&format!(
            "mcp_requests_failed {}\n",
            metrics.request.failed_requests
        ));

        output
            .push_str("# HELP mcp_request_duration_average Average request duration in seconds\n");
        output.push_str("# TYPE mcp_request_duration_average gauge\n");
        output.push_str(&format!(
            "mcp_request_duration_average {}\n",
            metrics.request.average_response_time
        ));

        // Transport metrics
        output.push_str("# HELP mcp_transport_bytes_sent Total bytes sent\n");
        output.push_str("# TYPE mcp_transport_bytes_sent counter\n");
        output.push_str(&format!(
            "mcp_transport_bytes_sent {}\n",
            metrics.transport.bytes_sent
        ));

        output.push_str("# HELP mcp_transport_bytes_received Total bytes received\n");
        output.push_str("# TYPE mcp_transport_bytes_received counter\n");
        output.push_str(&format!(
            "mcp_transport_bytes_received {}\n",
            metrics.transport.bytes_received
        ));

        // System metrics
        output.push_str("# HELP mcp_system_memory_usage Memory usage in bytes\n");
        output.push_str("# TYPE mcp_system_memory_usage gauge\n");
        output.push_str(&format!(
            "mcp_system_memory_usage {}\n",
            metrics.system.memory_usage
        ));

        output.push_str("# HELP mcp_system_cpu_usage CPU usage percentage\n");
        output.push_str("# TYPE mcp_system_cpu_usage gauge\n");
        output.push_str(&format!(
            "mcp_system_cpu_usage {}\n",
            metrics.system.cpu_usage
        ));

        output.push_str("# HELP mcp_system_active_connections Active connections\n");
        output.push_str("# TYPE mcp_system_active_connections gauge\n");
        output.push_str(&format!(
            "mcp_system_active_connections {}\n",
            metrics.system.active_connections
        ));

        output
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

// Default implementations
impl Default for RequestMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            method_counts: HashMap::new(),
        }
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            memory_usage: 0,
            cpu_usage: 0.0,
            active_connections: 0,
            uptime: Duration::new(0, 0),
        }
    }
}

/// Request timer for measuring request duration
pub struct RequestTimer {
    start: Instant,
    method: String,
    metrics: Arc<MetricsCollector>,
}

impl RequestTimer {
    pub fn start(method: impl Into<String>, metrics: Arc<MetricsCollector>) -> Self {
        Self {
            start: Instant::now(),
            method: method.into(),
            metrics,
        }
    }

    pub async fn finish(self, success: bool) {
        let duration = self.start.elapsed();
        self.metrics
            .record_request(&self.method, duration, success)
            .await;
    }
}
