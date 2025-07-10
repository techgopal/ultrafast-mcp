//! Metric and trace exporters for UltraFast MCP
//!
//! This module provides exporters for metrics and traces to various backends
//! including Prometheus, JSON, and custom formats.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::metrics::MetricsCollector;

/// Base trait for all exporters
#[async_trait::async_trait]
pub trait Exporter: Send + Sync {
    /// Export data to the configured destination
    async fn export(&self, data: &str) -> anyhow::Result<()>;
    
    /// Get the name of this exporter
    fn name(&self) -> &str;
    
    /// Check if the exporter is enabled
    fn is_enabled(&self) -> bool;
}

/// JSON exporter for metrics and traces
pub struct JsonExporter {
    name: String,
    enabled: bool,
    output_path: Option<String>,
}

impl JsonExporter {
    /// Create a new JSON exporter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            output_path: None,
        }
    }

    /// Create a new JSON exporter with file output
    pub fn with_file(name: impl Into<String>, output_path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            output_path: Some(output_path.into()),
        }
    }

    /// Set whether the exporter is enabled
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[async_trait::async_trait]
impl Exporter for JsonExporter {
    async fn export(&self, data: &str) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        if let Some(path) = &self.output_path {
            // Write to file
            tokio::fs::write(path, data).await?;
            debug!("Exported JSON data to file: {}", path);
        } else {
            // Write to stdout
            println!("{}", data);
            debug!("Exported JSON data to stdout");
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Prometheus exporter for metrics
pub struct PrometheusExporter {
    name: String,
    enabled: bool,
    endpoint: String,
    port: u16,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            endpoint: "127.0.0.1".to_string(),
            port: 9090,
        }
    }

    /// Create a new Prometheus exporter with custom endpoint
    pub fn with_endpoint(
        name: impl Into<String>,
        endpoint: impl Into<String>,
        port: u16,
    ) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            endpoint: endpoint.into(),
            port,
        }
    }

    /// Set whether the exporter is enabled
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Get the full endpoint URL
    pub fn endpoint_url(&self) -> String {
        format!("{}:{}", self.endpoint, self.port)
    }
}

#[async_trait::async_trait]
impl Exporter for PrometheusExporter {
    async fn export(&self, data: &str) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // For now, just log the Prometheus metrics
        // In a full implementation, this would start an HTTP server
        // and serve the metrics at the configured endpoint
        info!("Prometheus metrics available at {}:{}", self.endpoint, self.port);
        debug!("Prometheus metrics data: {}", data);

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Console exporter for development and debugging
pub struct ConsoleExporter {
    name: String,
    enabled: bool,
    format: ConsoleFormat,
}

/// Console output format
#[derive(Debug, Clone)]
pub enum ConsoleFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// Compact format
    Compact,
}

impl Default for ConsoleFormat {
    fn default() -> Self {
        Self::Human
    }
}

impl ConsoleExporter {
    /// Create a new console exporter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            format: ConsoleFormat::default(),
        }
    }

    /// Create a new console exporter with custom format
    pub fn with_format(name: impl Into<String>, format: ConsoleFormat) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            format,
        }
    }

    /// Set whether the exporter is enabled
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Format data according to the configured format
    fn format_data(&self, data: &str) -> String {
        match self.format {
            ConsoleFormat::Human => format!("[{}] {}", self.name, data),
            ConsoleFormat::Json => data.to_string(),
            ConsoleFormat::Compact => data.replace('\n', " ").trim().to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Exporter for ConsoleExporter {
    async fn export(&self, data: &str) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let formatted = self.format_data(data);
        println!("{}", formatted);
        debug!("Exported data to console: {}", formatted);

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Exporter manager for coordinating multiple exporters
pub struct ExporterManager {
    exporters: Arc<RwLock<Vec<Box<dyn Exporter>>>>,
    metrics_collector: Arc<MetricsCollector>,
}

impl ExporterManager {
    /// Create a new exporter manager
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            exporters: Arc::new(RwLock::new(Vec::new())),
            metrics_collector,
        }
    }

    /// Add an exporter
    pub async fn add_exporter(&self, exporter: Box<dyn Exporter>) {
        let mut exporters = self.exporters.write().await;
        exporters.push(exporter);
        info!("Added exporter: {}", exporters.last().unwrap().name());
    }

    /// Remove an exporter by name
    pub async fn remove_exporter(&self, name: &str) {
        let mut exporters = self.exporters.write().await;
        exporters.retain(|exporter| exporter.name() != name);
        info!("Removed exporter: {}", name);
    }

    /// Get all exporter names
    pub async fn get_exporter_names(&self) -> Vec<String> {
        let exporters = self.exporters.read().await;
        exporters.iter().map(|e| e.name().to_string()).collect()
    }

    /// Export metrics to all enabled exporters
    pub async fn export_metrics(&self) -> anyhow::Result<()> {
        let metrics = self.metrics_collector.get_metrics().await;
        let json_data = serde_json::to_string_pretty(&metrics)?;
        let prometheus_data = self.metrics_collector.export_prometheus().await;

        let exporters = self.exporters.read().await;
        
        for exporter in exporters.iter() {
            if !exporter.is_enabled() {
                continue;
            }

            let data = if exporter.name().contains("prometheus") {
                &prometheus_data
            } else {
                &json_data
            };

            match exporter.export(data).await {
                Ok(_) => debug!("Successfully exported to {}", exporter.name()),
                Err(e) => error!("Failed to export to {}: {}", exporter.name(), e),
            }
        }

        Ok(())
    }

    /// Export custom data to all enabled exporters
    pub async fn export_data(&self, data: &str) -> anyhow::Result<()> {
        let exporters = self.exporters.read().await;
        
        for exporter in exporters.iter() {
            if !exporter.is_enabled() {
                continue;
            }

            match exporter.export(data).await {
                Ok(_) => debug!("Successfully exported data to {}", exporter.name()),
                Err(e) => error!("Failed to export data to {}: {}", exporter.name(), e),
            }
        }

        Ok(())
    }

    /// Start periodic export task
    pub async fn start_periodic_export(&self, interval: std::time::Duration) -> tokio::task::JoinHandle<()> {
        let manager = self.clone();
        
        tokio::spawn(async move {
            loop {
                if let Err(e) = manager.export_metrics().await {
                    error!("Failed to export metrics: {}", e);
                }
                
                tokio::time::sleep(interval).await;
            }
        })
    }
}

impl Clone for ExporterManager {
    fn clone(&self) -> Self {
        Self {
            exporters: self.exporters.clone(),
            metrics_collector: self.metrics_collector.clone(),
        }
    }
}

/// Exporter configuration
#[derive(Debug, Clone)]
pub struct ExporterConfig {
    pub enabled: bool,
    pub export_interval: std::time::Duration,
    pub exporters: Vec<ExporterType>,
}

/// Types of exporters
#[derive(Debug, Clone)]
pub enum ExporterType {
    Console(ConsoleFormat),
    Json { output_path: Option<String> },
    Prometheus { endpoint: String, port: u16 },
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            export_interval: std::time::Duration::from_secs(60),
            exporters: vec![ExporterType::Console(ConsoleFormat::Human)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn test_json_exporter() {
        let exporter = JsonExporter::new("test");
        assert!(exporter.is_enabled());
        assert_eq!(exporter.name(), "test");

        // Test export to stdout
        let result = exporter.export("{\"test\": \"data\"}").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prometheus_exporter() {
        let exporter = PrometheusExporter::new("test");
        assert!(exporter.is_enabled());
        assert_eq!(exporter.name(), "test");
        assert_eq!(exporter.endpoint_url(), "127.0.0.1:9090");

        let result = exporter.export("# HELP test_metric Test metric").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_exporter() {
        let exporter = ConsoleExporter::new("test");
        assert!(exporter.is_enabled());
        assert_eq!(exporter.name(), "test");

        let result = exporter.export("test data").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exporter_manager() {
        let metrics_collector = Arc::new(MetricsCollector::new());
        let manager = ExporterManager::new(metrics_collector);

        // Add exporters
        let console_exporter = Box::new(ConsoleExporter::new("console"));
        let json_exporter = Box::new(JsonExporter::new("json"));
        
        manager.add_exporter(console_exporter).await;
        manager.add_exporter(json_exporter).await;

        let names = manager.get_exporter_names().await;
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"console".to_string()));
        assert!(names.contains(&"json".to_string()));

        // Test export
        let result = manager.export_metrics().await;
        assert!(result.is_ok());

        // Remove exporter
        manager.remove_exporter("console").await;
        let names = manager.get_exporter_names().await;
        assert_eq!(names.len(), 1);
        assert!(!names.contains(&"console".to_string()));
    }

    #[tokio::test]
    async fn test_exporter_config() {
        let config = ExporterConfig::default();
        assert!(config.enabled);
        assert_eq!(config.export_interval, std::time::Duration::from_secs(60));
        assert_eq!(config.exporters.len(), 1);

        if let ExporterType::Console(format) = &config.exporters[0] {
            assert!(matches!(format, ConsoleFormat::Human));
        } else {
            panic!("Expected Console exporter type");
        }
    }
}
