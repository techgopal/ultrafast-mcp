//! Health checking and monitoring for Ultrafast MCP

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded but functional
    Degraded(String),
    /// Service is unhealthy
    Unhealthy(String),
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "HEALTHY"),
            HealthStatus::Degraded(msg) => write!(f, "DEGRADED: {msg}"),
            HealthStatus::Unhealthy(msg) => write!(f, "UNHEALTHY: {msg}"),
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub duration: Duration,
    pub timestamp: SystemTime,
}

/// Health check trait for custom health checks
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Name of the health check
    fn name(&self) -> &str;

    /// Perform the health check
    async fn check(&self) -> HealthCheckResult;
}

/// Main health checker
pub struct HealthChecker {
    checks: Arc<RwLock<Vec<Box<dyn HealthCheck>>>>,
    last_results: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    start_time: Instant,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(Vec::new())),
            last_results: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Add a health check
    pub async fn add_check(&self, check: Box<dyn HealthCheck>) {
        let mut checks = self.checks.write().await;
        checks.push(check);
    }

    /// Run all health checks
    pub async fn check_all(&self) -> HealthStatus {
        let checks = self.checks.read().await;
        let mut degraded_reasons = Vec::new();
        let mut unhealthy_reasons = Vec::new();

        for check in checks.iter() {
            let result = check.check().await;
            let name = check.name();

            // Store the result
            {
                let mut results = self.last_results.write().await;
                results.insert(name.to_string(), result.clone());
            }

            match result.status {
                HealthStatus::Healthy => {}
                HealthStatus::Degraded(reason) => {
                    degraded_reasons.push(format!("{name}: {reason}"));
                }
                HealthStatus::Unhealthy(reason) => {
                    unhealthy_reasons.push(format!("{name}: {reason}"));
                }
            }
        }

        if !unhealthy_reasons.is_empty() {
            HealthStatus::Unhealthy(unhealthy_reasons.join(", "))
        } else if !degraded_reasons.is_empty() {
            HealthStatus::Degraded(degraded_reasons.join(", "))
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get the last results for all checks
    pub async fn last_results(&self) -> HashMap<String, HealthCheckResult> {
        self.last_results.read().await.clone()
    }

    /// Get uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic system health check
pub struct SystemHealthCheck {
    name: String,
}

impl SystemHealthCheck {
    pub fn new() -> Self {
        Self {
            name: "system".to_string(),
        }
    }
}

impl Default for SystemHealthCheck {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl HealthCheck for SystemHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();

        // Basic system checks - always pass for now
        let status = HealthStatus::Healthy;

        HealthCheckResult {
            status,
            duration: start.elapsed(),
            timestamp: SystemTime::now(),
        }
    }
}

/// Memory health check
pub struct MemoryHealthCheck {
    name: String,
    threshold: f64, // Memory usage threshold (0.0-1.0)
}

impl MemoryHealthCheck {
    pub fn new(threshold: f64) -> Self {
        Self {
            name: "memory".to_string(),
            threshold,
        }
    }

    /// Get current memory usage as a percentage (0.0-1.0)
    fn get_memory_usage(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // Simplified memory check - in a real implementation,
        // you'd use system APIs to get actual memory usage
        // For now, return a fallback value
        Ok(0.3)
    }


}

#[async_trait::async_trait]
impl HealthCheck for MemoryHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();

        // Get system memory info (simplified implementation)
        let status = match self.get_memory_usage() {
            Ok(usage) if usage > self.threshold => HealthStatus::Unhealthy(format!(
                "Memory usage {:.1}% exceeds threshold {:.1}%",
                usage * 100.0,
                self.threshold * 100.0
            )),
            Ok(_) => HealthStatus::Healthy,
            Err(e) => HealthStatus::Unhealthy(format!("Failed to get memory usage: {e}")),
        };

        HealthCheckResult {
            status,
            duration: start.elapsed(),
            timestamp: SystemTime::now(),
        }
    }
}
