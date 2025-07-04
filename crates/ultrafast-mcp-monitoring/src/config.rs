//! Configuration for monitoring and observability

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Main monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringConfig {
    /// Tracing configuration
    pub tracing: TracingConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
    /// Health check configuration
    pub health: HealthConfig,
    /// HTTP server configuration for metrics endpoints
    pub http: HttpConfig,
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable tracing
    pub enabled: bool,
    /// Service name for tracing
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Environment (dev, staging, prod)
    pub environment: String,
    /// Jaeger configuration
    pub jaeger: Option<JaegerConfig>,
    /// OTLP configuration
    pub otlp: Option<OtlpConfig>,
    /// Console exporter for development
    pub console: bool,
    /// Log level filter
    pub level: String,
    /// Sample rate (0.0 to 1.0)
    pub sample_rate: f32,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Prometheus configuration
    pub prometheus: Option<PrometheusConfig>,
    /// OTLP metrics configuration
    pub otlp: Option<OtlpConfig>,
    /// Collection interval for system metrics
    pub collection_interval: Duration,
    /// Enable system metrics
    pub system_metrics: bool,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval
    pub check_interval: Duration,
    /// Timeout for health checks
    pub timeout: Duration,
    /// Custom health checks
    pub custom_checks: Vec<String>,
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Enable HTTP server
    pub enabled: bool,
    /// Bind address
    pub address: String,
    /// Port
    pub port: u16,
    /// Enable TLS
    pub tls: Option<TlsConfig>,
}

/// Jaeger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerConfig {
    /// Jaeger agent endpoint
    pub agent_endpoint: String,
    /// Jaeger collector endpoint
    pub collector_endpoint: Option<String>,
    /// Authentication headers
    pub headers: HashMap<String, String>,
}

/// OTLP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpConfig {
    /// OTLP endpoint
    pub endpoint: String,
    /// Authentication headers
    pub headers: HashMap<String, String>,
    /// Timeout
    pub timeout: Duration,
    /// Compression
    pub compression: Option<String>,
}

/// Prometheus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    /// Registry name
    pub registry_name: String,
    /// Metrics prefix
    pub prefix: String,
    /// Custom labels
    pub labels: HashMap<String, String>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate file path
    pub cert_file: String,
    /// Private key file path
    pub key_file: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_name: "ultrafast-mcp".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
            jaeger: Some(JaegerConfig::default()),
            otlp: None,
            console: true,
            level: "info".to_string(),
            sample_rate: 1.0,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prometheus: Some(PrometheusConfig::default()),
            otlp: None,
            collection_interval: Duration::from_secs(30),
            system_metrics: true,
        }
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            custom_checks: vec![],
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            address: "127.0.0.1".to_string(),
            port: 9090,
            tls: None,
        }
    }
}

impl Default for JaegerConfig {
    fn default() -> Self {
        Self {
            agent_endpoint: "http://localhost:14268/api/traces".to_string(),
            collector_endpoint: None,
            headers: HashMap::new(),
        }
    }
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            registry_name: "ultrafast_mcp".to_string(),
            prefix: "mcp".to_string(),
            labels: HashMap::new(),
        }
    }
}

impl MonitoringConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Override with environment variables
        if let Ok(val) = std::env::var("MCP_TRACING_ENABLED") {
            config.tracing.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var("MCP_SERVICE_NAME") {
            config.tracing.service_name = val;
        }

        if let Ok(val) = std::env::var("MCP_ENVIRONMENT") {
            config.tracing.environment = val;
        }

        if let Ok(val) = std::env::var("MCP_LOG_LEVEL") {
            config.tracing.level = val;
        }

        if let Ok(val) = std::env::var("MCP_JAEGER_ENDPOINT") {
            if let Some(ref mut jaeger) = config.tracing.jaeger {
                jaeger.agent_endpoint = val;
            }
        }

        if let Ok(val) = std::env::var("MCP_METRICS_ENABLED") {
            config.metrics.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var("MCP_HTTP_PORT") {
            config.http.port = val.parse().unwrap_or(9090);
        }

        config
    }

    /// Load configuration from a file
    #[cfg(feature = "config-files")]
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = match path.split('.').next_back() {
            Some("toml") => toml::from_str(&content)?,
            #[cfg(feature = "config-files")]
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
            Some("json") => serde_json::from_str(&content)?,
            _ => return Err(anyhow::anyhow!("Unsupported file format")),
        };
        Ok(config)
    }

    /// Load configuration from a file (no-op without config-files feature)
    #[cfg(not(feature = "config-files"))]
    pub fn from_file(_path: &str) -> anyhow::Result<Self> {
        Err(anyhow::anyhow!(
            "Config file support not enabled. Enable 'config-files' feature."
        ))
    }
}
