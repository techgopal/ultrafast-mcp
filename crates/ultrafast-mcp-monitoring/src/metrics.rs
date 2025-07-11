//! Metrics collection and management for UltraFast MCP
//!
//! This module provides comprehensive metrics collection for MCP servers and clients,
//! including request metrics, transport metrics, and system metrics.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Core metrics structure containing all collected metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct Metrics {
    pub request: RequestMetrics,
    pub transport: TransportMetrics,
    pub system: SystemMetrics,
}

/// Request-related metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct RequestMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub method_counts: HashMap<String, u64>,
    pub response_time_histogram: HashMap<String, Vec<Duration>>,
    pub last_request_time: Option<SystemTime>,
}

/// Transport-related metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct TransportMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_count: u32,
    pub error_count: u64,
    pub active_connections: u32,
    pub connection_errors: HashMap<String, u64>,
    pub last_activity: Option<SystemTime>,
}

/// System-related metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemMetrics {
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub active_connections: u32,
    pub uptime: Duration,
    pub start_time: SystemTime,
    pub last_update: SystemTime,
}

/// Metrics collector for gathering and managing metrics
pub struct MetricsCollector {
    metrics: Arc<RwLock<Metrics>>,
    collection_interval: Duration,
    max_histogram_size: usize,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Metrics {
                request: RequestMetrics::default(),
                transport: TransportMetrics::default(),
                system: SystemMetrics::default(),
            })),
            collection_interval: Duration::from_secs(30),
            max_histogram_size: 1000,
        }
    }

    /// Create a new metrics collector with custom configuration
    pub fn with_config(collection_interval: Duration, max_histogram_size: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Metrics {
                request: RequestMetrics::default(),
                transport: TransportMetrics::default(),
                system: SystemMetrics::default(),
            })),
            collection_interval,
            max_histogram_size,
        }
    }

    /// Record a request with timing and success status
    pub async fn record_request(&self, method: &str, response_time: Duration, success: bool) {
        let mut metrics = self.metrics.write().await;

        // Update request counts
        metrics.request.total_requests += 1;
        if success {
            metrics.request.successful_requests += 1;
        } else {
            metrics.request.failed_requests += 1;
        }

        // Update method counts
        *metrics
            .request
            .method_counts
            .entry(method.to_string())
            .or_insert(0) += 1;

        // Update response time histogram
        let method_histogram = metrics
            .request
            .response_time_histogram
            .entry(method.to_string())
            .or_insert_with(Vec::new);

        method_histogram.push(response_time);

        // Keep histogram size manageable
        if method_histogram.len() > self.max_histogram_size {
            method_histogram.remove(0);
        }

        // Update average response time
        let total_time: Duration = method_histogram.iter().sum();
        metrics.request.average_response_time =
            total_time.as_millis() as f64 / method_histogram.len() as f64;

        // Update last request time
        metrics.request.last_request_time = Some(SystemTime::now());

        debug!(
            "Recorded request: method={}, response_time={:?}, success={}",
            method, response_time, success
        );
    }

    /// Record transport send operation
    pub async fn record_transport_send(&self, bytes: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.transport.bytes_sent += bytes;
        metrics.transport.last_activity = Some(SystemTime::now());

        debug!("Recorded transport send: {} bytes", bytes);
    }

    /// Record transport receive operation
    pub async fn record_transport_receive(&self, bytes: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.transport.bytes_received += bytes;
        metrics.transport.last_activity = Some(SystemTime::now());

        debug!("Recorded transport receive: {} bytes", bytes);
    }

    /// Record transport error
    pub async fn record_transport_error(&self, error_type: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.transport.error_count += 1;
        *metrics
            .transport
            .connection_errors
            .entry(error_type.to_string())
            .or_insert(0) += 1;

        warn!("Recorded transport error: {}", error_type);
    }

    /// Update connection count
    pub async fn update_connection_count(&self, count: u32) {
        let mut metrics = self.metrics.write().await;
        metrics.transport.connection_count = count;
        metrics.transport.active_connections = count;

        debug!("Updated connection count: {}", count);
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
        metrics.system.last_update = SystemTime::now();

        // Update uptime
        if let Ok(elapsed) = metrics.system.start_time.elapsed() {
            metrics.system.uptime = elapsed;
        }

        debug!(
            "Updated system metrics: connections={}, memory={}, cpu={}%",
            active_connections, memory_usage, cpu_usage
        );
    }

    /// Get current metrics snapshot
    pub async fn get_metrics(&self) -> Metrics {
        self.metrics.read().await.clone()
    }

    /// Export metrics as JSON
    pub async fn export_json(&self) -> serde_json::Result<String> {
        let metrics = self.get_metrics().await;
        serde_json::to_string_pretty(&metrics)
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let metrics = self.get_metrics().await;
        let mut prometheus_output = String::new();

        // Request metrics
        prometheus_output.push_str("# HELP mcp_requests_total Total number of requests\n");
        prometheus_output.push_str("# TYPE mcp_requests_total counter\n");
        prometheus_output.push_str(&format!(
            "mcp_requests_total {}\n",
            metrics.request.total_requests
        ));

        prometheus_output
            .push_str("# HELP mcp_requests_successful Total number of successful requests\n");
        prometheus_output.push_str("# TYPE mcp_requests_successful counter\n");
        prometheus_output.push_str(&format!(
            "mcp_requests_successful {}\n",
            metrics.request.successful_requests
        ));

        prometheus_output.push_str("# HELP mcp_requests_failed Total number of failed requests\n");
        prometheus_output.push_str("# TYPE mcp_requests_failed counter\n");
        prometheus_output.push_str(&format!(
            "mcp_requests_failed {}\n",
            metrics.request.failed_requests
        ));

        prometheus_output.push_str(
            "# HELP mcp_request_duration_average Average request duration in milliseconds\n",
        );
        prometheus_output.push_str("# TYPE mcp_request_duration_average gauge\n");
        prometheus_output.push_str(&format!(
            "mcp_request_duration_average {}\n",
            metrics.request.average_response_time
        ));

        // Method-specific metrics
        for (method, count) in &metrics.request.method_counts {
            prometheus_output.push_str(
                "# HELP mcp_requests_by_method_total Total requests by method\n"
                    .to_string()
                    .as_str(),
            );
            prometheus_output.push_str(
                "# TYPE mcp_requests_by_method_total counter\n"
                    .to_string()
                    .as_str(),
            );
            prometheus_output.push_str(&format!(
                "mcp_requests_by_method_total{{method=\"{}\"}} {}\n",
                method, count
            ));
        }

        // Transport metrics
        prometheus_output.push_str("# HELP mcp_transport_bytes_sent Total bytes sent\n");
        prometheus_output.push_str("# TYPE mcp_transport_bytes_sent counter\n");
        prometheus_output.push_str(&format!(
            "mcp_transport_bytes_sent {}\n",
            metrics.transport.bytes_sent
        ));

        prometheus_output.push_str("# HELP mcp_transport_bytes_received Total bytes received\n");
        prometheus_output.push_str("# TYPE mcp_transport_bytes_received counter\n");
        prometheus_output.push_str(&format!(
            "mcp_transport_bytes_received {}\n",
            metrics.transport.bytes_received
        ));

        prometheus_output
            .push_str("# HELP mcp_transport_connections_active Current active connections\n");
        prometheus_output.push_str("# TYPE mcp_transport_connections_active gauge\n");
        prometheus_output.push_str(&format!(
            "mcp_transport_connections_active {}\n",
            metrics.transport.active_connections
        ));

        prometheus_output.push_str("# HELP mcp_transport_errors_total Total transport errors\n");
        prometheus_output.push_str("# TYPE mcp_transport_errors_total counter\n");
        prometheus_output.push_str(&format!(
            "mcp_transport_errors_total {}\n",
            metrics.transport.error_count
        ));

        // System metrics
        prometheus_output
            .push_str("# HELP mcp_system_memory_usage_bytes Current memory usage in bytes\n");
        prometheus_output.push_str("# TYPE mcp_system_memory_usage_bytes gauge\n");
        prometheus_output.push_str(&format!(
            "mcp_system_memory_usage_bytes {}\n",
            metrics.system.memory_usage
        ));

        prometheus_output
            .push_str("# HELP mcp_system_cpu_usage_percent Current CPU usage percentage\n");
        prometheus_output.push_str("# TYPE mcp_system_cpu_usage_percent gauge\n");
        prometheus_output.push_str(&format!(
            "mcp_system_cpu_usage_percent {}\n",
            metrics.system.cpu_usage
        ));

        prometheus_output.push_str("# HELP mcp_system_uptime_seconds System uptime in seconds\n");
        prometheus_output.push_str("# TYPE mcp_system_uptime_seconds gauge\n");
        prometheus_output.push_str(&format!(
            "mcp_system_uptime_seconds {}\n",
            metrics.system.uptime.as_secs()
        ));

        prometheus_output
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = Metrics {
            request: RequestMetrics::default(),
            transport: TransportMetrics::default(),
            system: SystemMetrics::default(),
        };

        info!("Metrics reset completed");
    }

    /// Get metrics collection interval
    pub fn collection_interval(&self) -> Duration {
        self.collection_interval
    }

    /// Set metrics collection interval
    pub fn set_collection_interval(&mut self, interval: Duration) {
        self.collection_interval = interval;
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RequestMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            method_counts: HashMap::new(),
            response_time_histogram: HashMap::new(),
            last_request_time: None,
        }
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            memory_usage: 0,
            cpu_usage: 0.0,
            active_connections: 0,
            uptime: Duration::ZERO,
            start_time: SystemTime::now(),
            last_update: SystemTime::now(),
        }
    }
}

/// Timer for measuring request duration
pub struct RequestTimer {
    start: Instant,
    method: String,
    metrics: Arc<MetricsCollector>,
}

impl RequestTimer {
    /// Start a new request timer
    pub fn start(method: impl Into<String>, metrics: Arc<MetricsCollector>) -> Self {
        Self {
            start: Instant::now(),
            method: method.into(),
            metrics,
        }
    }

    /// Finish the timer and record the metrics
    pub async fn finish(self, success: bool) {
        let duration = self.start.elapsed();
        self.metrics
            .record_request(&self.method, duration, success)
            .await;

        debug!(
            "Request completed: method={}, duration={:?}, success={}",
            self.method, duration, success
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        let metrics = collector.get_metrics().await;

        assert_eq!(metrics.request.total_requests, 0);
        assert_eq!(metrics.transport.bytes_sent, 0);
        assert_eq!(metrics.system.memory_usage, 0);
    }

    #[tokio::test]
    async fn test_request_recording() {
        let collector = Arc::new(MetricsCollector::new());

        // Record a successful request
        collector
            .record_request("test_method", Duration::from_millis(100), true)
            .await;

        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.request.total_requests, 1);
        assert_eq!(metrics.request.successful_requests, 1);
        assert_eq!(metrics.request.failed_requests, 0);
        assert_eq!(metrics.request.method_counts["test_method"], 1);
    }

    #[tokio::test]
    async fn test_transport_metrics() {
        let collector = Arc::new(MetricsCollector::new());

        collector.record_transport_send(1024).await;
        collector.record_transport_receive(2048).await;
        collector.record_transport_error("connection_failed").await;

        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.transport.bytes_sent, 1024);
        assert_eq!(metrics.transport.bytes_received, 2048);
        assert_eq!(metrics.transport.error_count, 1);
        assert_eq!(metrics.transport.connection_errors["connection_failed"], 1);
    }

    #[tokio::test]
    async fn test_request_timer() {
        let collector = Arc::new(MetricsCollector::new());

        let timer = RequestTimer::start("timer_test", collector.clone());
        sleep(Duration::from_millis(10)).await;
        timer.finish(true).await;

        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.request.total_requests, 1);
        assert_eq!(metrics.request.successful_requests, 1);
        assert!(metrics.request.average_response_time > 0.0);
    }

    #[tokio::test]
    async fn test_prometheus_export() {
        let collector = Arc::new(MetricsCollector::new());

        collector
            .record_request("test", Duration::from_millis(50), true)
            .await;
        collector.record_transport_send(100).await;

        let prometheus_output = collector.export_prometheus().await;

        assert!(prometheus_output.contains("mcp_requests_total 1"));
        assert!(prometheus_output.contains("mcp_transport_bytes_sent 100"));
        assert!(prometheus_output.contains("mcp_request_duration_average"));
    }

    #[tokio::test]
    async fn test_metrics_reset() {
        let collector = Arc::new(MetricsCollector::new());

        collector
            .record_request("test", Duration::from_millis(50), true)
            .await;
        collector.record_transport_send(100).await;

        // Verify metrics were recorded
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.request.total_requests, 1);
        assert_eq!(metrics.transport.bytes_sent, 100);

        // Reset metrics
        collector.reset().await;

        // Verify metrics were reset
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.request.total_requests, 0);
        assert_eq!(metrics.transport.bytes_sent, 0);
    }
}
