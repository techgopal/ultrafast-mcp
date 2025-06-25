//! Ultrafast MCP Monitoring Library
//!
//! This crate provides comprehensive monitoring and observability for the Ultrafast MCP framework,
//! including metrics collection, health checking, tracing, and OpenTelemetry integration.

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use axum::{
    routing::get,
    Router,
};

pub mod config;
pub mod health;
pub mod metrics;
pub mod middleware;
pub mod tracing;
pub mod exporters;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        health_checker.add_check(Box::new(health::SystemHealthCheck::new())).await;

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
        self.health_checker.add_check(Box::new(health::SystemHealthCheck::new())).await;
        Ok(())
    }

    /// Start the HTTP monitoring server
    pub async fn start_http_server(&self, addr: std::net::SocketAddr) -> Result<()> {
        let app = Router::new()
            .route("/metrics", get({
                let collector = self.metrics_collector.clone();
                move || async move { collector.export_prometheus().await }
            }))
            .route("/health", get({
                let health = self.health_checker.clone();
                move || async move { 
                    match health.check_all().await {
                        HealthStatus::Healthy => "OK",
                        HealthStatus::Unhealthy(_) => "UNHEALTHY",
                        HealthStatus::Degraded(_) => "DEGRADED",
                    }
                }
            }))
            .route("/metrics/json", get({
                let collector = self.metrics_collector.clone();
                move || async move {
                    match collector.export_json().await {
                        Ok(json) => json,
                        Err(_) => "{}".to_string(),
                    }
                }
            }));

        println!("Starting monitoring HTTP server on {}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
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
        *metrics.request.method_counts.entry(method.to_string()).or_insert(0) += 1;
        
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
    pub async fn update_system_metrics(&self, active_connections: u32, memory_usage: u64, cpu_usage: f64) {
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
        output.push_str(&format!("# HELP mcp_requests_total Total number of requests\n"));
        output.push_str(&format!("# TYPE mcp_requests_total counter\n"));
        output.push_str(&format!("mcp_requests_total {}\n", metrics.request.total_requests));
        
        output.push_str(&format!("# HELP mcp_requests_successful Total number of successful requests\n"));
        output.push_str(&format!("# TYPE mcp_requests_successful counter\n"));
        output.push_str(&format!("mcp_requests_successful {}\n", metrics.request.successful_requests));
        
        output.push_str(&format!("# HELP mcp_requests_failed Total number of failed requests\n"));
        output.push_str(&format!("# TYPE mcp_requests_failed counter\n"));
        output.push_str(&format!("mcp_requests_failed {}\n", metrics.request.failed_requests));
        
        output.push_str(&format!("# HELP mcp_request_duration_average Average request duration in seconds\n"));
        output.push_str(&format!("# TYPE mcp_request_duration_average gauge\n"));
        output.push_str(&format!("mcp_request_duration_average {}\n", metrics.request.average_response_time));
        
        // Transport metrics
        output.push_str(&format!("# HELP mcp_transport_bytes_sent Total bytes sent\n"));
        output.push_str(&format!("# TYPE mcp_transport_bytes_sent counter\n"));
        output.push_str(&format!("mcp_transport_bytes_sent {}\n", metrics.transport.bytes_sent));
        
        output.push_str(&format!("# HELP mcp_transport_bytes_received Total bytes received\n"));
        output.push_str(&format!("# TYPE mcp_transport_bytes_received counter\n"));
        output.push_str(&format!("mcp_transport_bytes_received {}\n", metrics.transport.bytes_received));
        
        // System metrics
        output.push_str(&format!("# HELP mcp_system_memory_usage Memory usage in bytes\n"));
        output.push_str(&format!("# TYPE mcp_system_memory_usage gauge\n"));
        output.push_str(&format!("mcp_system_memory_usage {}\n", metrics.system.memory_usage));
        
        output.push_str(&format!("# HELP mcp_system_cpu_usage CPU usage percentage\n"));
        output.push_str(&format!("# TYPE mcp_system_cpu_usage gauge\n"));
        output.push_str(&format!("mcp_system_cpu_usage {}\n", metrics.system.cpu_usage));
        
        output.push_str(&format!("# HELP mcp_system_active_connections Active connections\n"));
        output.push_str(&format!("# TYPE mcp_system_active_connections gauge\n"));
        output.push_str(&format!("mcp_system_active_connections {}\n", metrics.system.active_connections));
        
        output
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

impl Default for TransportMetrics {
    fn default() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            connection_count: 0,
            error_count: 0,
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
        self.metrics.record_request(&self.method, duration, success).await;
    }
}
