//! Health checking and status monitoring for UltraFast MCP
//!
//! This module provides comprehensive health checking capabilities for MCP servers
//! and clients, including system health, application health, and custom health checks.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy and operating normally
    Healthy,
    /// System is degraded but still functional
    Degraded(Vec<String>),
    /// System is unhealthy and requires attention
    Unhealthy(Vec<String>),
}

impl HealthStatus {
    /// Check if the status represents a healthy state
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Check if the status represents a degraded state
    pub fn is_degraded(&self) -> bool {
        matches!(self, HealthStatus::Degraded(_))
    }

    /// Check if the status represents an unhealthy state
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, HealthStatus::Unhealthy(_))
    }

    /// Get the severity level as a numeric value
    pub fn severity(&self) -> u8 {
        match self {
            HealthStatus::Healthy => 0,
            HealthStatus::Degraded(_) => 1,
            HealthStatus::Unhealthy(_) => 2,
        }
    }
}

/// Result of a health check
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub duration: Duration,
    pub timestamp: SystemTime,
    pub details: Option<String>,
}

/// Trait for implementing custom health checks
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform the health check
    async fn check(&self) -> HealthCheckResult;

    /// Get the name of this health check
    fn name(&self) -> &str;

    /// Get the timeout for this health check
    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    /// Get the interval between health checks
    fn interval(&self) -> Duration {
        Duration::from_secs(60)
    }
}

/// System health check implementation
pub struct SystemHealthCheck {
    name: String,
    timeout: Duration,
    interval: Duration,
}

impl SystemHealthCheck {
    /// Create a new system health check
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            timeout: Duration::from_secs(30),
            interval: Duration::from_secs(60),
        }
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set custom interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
}

#[async_trait]
impl HealthCheck for SystemHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        // Basic system health check
        let status = match self.perform_system_check().await {
            Ok(_) => HealthStatus::Healthy,
            Err(e) => HealthStatus::Unhealthy(vec![e.to_string()]),
        };

        HealthCheckResult {
            status,
            duration: start.elapsed(),
            timestamp: SystemTime::now(),
            details: Some("System resources check completed".to_string()),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn timeout(&self) -> Duration {
        self.timeout
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}

impl SystemHealthCheck {
    async fn perform_system_check(&self) -> anyhow::Result<()> {
        // Check memory usage
        let mut memory_info = sysinfo::System::new_all();
        memory_info.refresh_memory();
        let memory_usage = memory_info.used_memory();
        let total_memory = memory_info.total_memory();
        let memory_percentage = (memory_usage as f64 / total_memory as f64) * 100.0;

        if memory_percentage > 90.0 {
            return Err(anyhow::anyhow!(
                "Memory usage too high: {:.1}%",
                memory_percentage
            ));
        }

        // Check disk space (basic check) - simplified for now
        // TODO: Implement proper disk space checking when sysinfo API is stable
        debug!("Disk space check skipped - API compatibility issue");

        Ok(())
    }
}

/// Application health check implementation
pub struct ApplicationHealthCheck {
    name: String,
    check_fn: Arc<dyn Fn() -> anyhow::Result<()> + Send + Sync>,
    timeout: Duration,
    interval: Duration,
}

impl ApplicationHealthCheck {
    /// Create a new application health check with a custom check function
    pub fn new<F>(name: impl Into<String>, check_fn: F) -> Self
    where
        F: Fn() -> anyhow::Result<()> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            check_fn: Arc::new(check_fn),
            timeout: Duration::from_secs(30),
            interval: Duration::from_secs(60),
        }
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set custom interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }
}

#[async_trait]
impl HealthCheck for ApplicationHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        let status = match (self.check_fn)() {
            Ok(_) => HealthStatus::Healthy,
            Err(e) => HealthStatus::Unhealthy(vec![e.to_string()]),
        };

        HealthCheckResult {
            status,
            duration: start.elapsed(),
            timestamp: SystemTime::now(),
            details: Some("Application health check completed".to_string()),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn timeout(&self) -> Duration {
        self.timeout
    }

    fn interval(&self) -> Duration {
        self.interval
    }
}

/// Health checker for managing multiple health checks
pub struct HealthChecker {
    checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck>>>>,
    results: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    last_check: Arc<RwLock<Option<SystemTime>>>,
    check_interval: Duration,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            last_check: Arc::new(RwLock::new(None)),
            check_interval: Duration::from_secs(60),
        }
    }

    /// Create a new health checker with custom interval
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            last_check: Arc::new(RwLock::new(None)),
            check_interval: interval,
        }
    }

    /// Add a health check
    pub async fn add_check(&self, check: Box<dyn HealthCheck>) {
        let name = check.name().to_string();
        let mut checks = self.checks.write().await;
        checks.insert(name.clone(), check);

        info!("Added health check: {}", name);
    }

    /// Remove a health check
    pub async fn remove_check(&self, name: &str) {
        let mut checks = self.checks.write().await;
        checks.remove(name);

        let mut results = self.results.write().await;
        results.remove(name);

        info!("Removed health check: {}", name);
    }

    /// Get all registered health check names
    pub async fn get_check_names(&self) -> Vec<String> {
        let checks = self.checks.read().await;
        checks.keys().cloned().collect()
    }

    /// Run a specific health check
    pub async fn run_check(&self, name: &str) -> Option<HealthCheckResult> {
        let checks = self.checks.read().await;
        if let Some(check) = checks.get(name) {
            let result = check.check().await;

            let mut results = self.results.write().await;
            results.insert(name.to_string(), result.clone());

            debug!("Health check '{}' completed: {:?}", name, result.status);
            Some(result)
        } else {
            warn!("Health check '{}' not found", name);
            None
        }
    }

    /// Run all health checks
    pub async fn run_all_checks(&self) -> HashMap<String, HealthCheckResult> {
        let checks = self.checks.read().await;
        let mut results = HashMap::new();

        for (name, check) in checks.iter() {
            let result = check.check().await;
            results.insert(name.clone(), result);
        }

        // Store results
        {
            let mut stored_results = self.results.write().await;
            *stored_results = results.clone();
        }

        // Update last check time
        {
            let mut last_check = self.last_check.write().await;
            *last_check = Some(SystemTime::now());
        }

        debug!("Completed {} health checks", results.len());
        results
    }

    /// Get the overall health status
    pub async fn get_overall_health(&self) -> HealthStatus {
        let results = self.results.read().await;

        if results.is_empty() {
            return HealthStatus::Healthy; // No checks means healthy by default
        }

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for (name, result) in results.iter() {
            match &result.status {
                HealthStatus::Unhealthy(messages) => {
                    errors.extend(messages.iter().map(|msg| format!("{}: {}", name, msg)));
                }
                HealthStatus::Degraded(messages) => {
                    warnings.extend(messages.iter().map(|msg| format!("{}: {}", name, msg)));
                }
                HealthStatus::Healthy => {}
            }
        }

        if !errors.is_empty() {
            HealthStatus::Unhealthy(errors)
        } else if !warnings.is_empty() {
            HealthStatus::Degraded(warnings)
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get the last check time
    pub async fn get_last_check_time(&self) -> Option<SystemTime> {
        let last_check = self.last_check.read().await;
        *last_check
    }

    /// Get all health check results
    pub async fn get_all_results(&self) -> HashMap<String, HealthCheckResult> {
        let results = self.results.read().await;
        results.clone()
    }

    /// Get a specific health check result
    pub async fn get_result(&self, name: &str) -> Option<HealthCheckResult> {
        let results = self.results.read().await;
        results.get(name).cloned()
    }

    /// Check if it's time to run health checks
    pub async fn should_run_checks(&self) -> bool {
        let last_check = self.last_check.read().await;
        match *last_check {
            Some(time) => {
                if let Ok(elapsed) = time.elapsed() {
                    elapsed >= self.check_interval
                } else {
                    true
                }
            }
            None => true,
        }
    }

    /// Start the health checker background task
    pub async fn start_background_task(&self) -> tokio::task::JoinHandle<()> {
        let checker = self.clone();

        tokio::spawn(async move {
            loop {
                if checker.should_run_checks().await {
                    checker.run_all_checks().await;
                }

                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        })
    }
}

impl Clone for HealthChecker {
    fn clone(&self) -> Self {
        Self {
            checks: self.checks.clone(),
            results: self.results.clone(),
            last_check: self.last_check.clone(),
            check_interval: self.check_interval,
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthConfig {
    pub enabled: bool,
    pub check_interval: Duration,
    pub timeout: Duration,
    pub background_task: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(60),
            timeout: Duration::from_secs(30),
            background_task: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_health_status() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Healthy.is_degraded());
        assert!(!HealthStatus::Healthy.is_unhealthy());
        assert_eq!(HealthStatus::Healthy.severity(), 0);

        assert!(!HealthStatus::Degraded(vec!["warning".to_string()]).is_healthy());
        assert!(HealthStatus::Degraded(vec!["warning".to_string()]).is_degraded());
        assert!(!HealthStatus::Degraded(vec!["warning".to_string()]).is_unhealthy());
        assert_eq!(
            HealthStatus::Degraded(vec!["warning".to_string()]).severity(),
            1
        );

        assert!(!HealthStatus::Unhealthy(vec!["error".to_string()]).is_healthy());
        assert!(!HealthStatus::Unhealthy(vec!["error".to_string()]).is_degraded());
        assert!(HealthStatus::Unhealthy(vec!["error".to_string()]).is_unhealthy());
        assert_eq!(
            HealthStatus::Unhealthy(vec!["error".to_string()]).severity(),
            2
        );
    }

    #[tokio::test]
    async fn test_application_health_check() {
        let check = ApplicationHealthCheck::new("test_check", || {
            // Simulate a successful check
            Ok(())
        });

        let result = check.check().await;
        assert!(result.status.is_healthy());
        assert!(result.duration.as_millis() < 100); // Should be very fast
    }

    #[tokio::test]
    async fn test_application_health_check_failure() {
        let check = ApplicationHealthCheck::new("test_check", || {
            // Simulate a failed check
            Err(anyhow::anyhow!("Test error"))
        });

        let result = check.check().await;
        assert!(result.status.is_unhealthy());
    }

    #[tokio::test]
    async fn test_health_checker() {
        let checker = HealthChecker::new();

        // Add a health check
        let check = ApplicationHealthCheck::new("test_check", || Ok(()));
        checker.add_check(Box::new(check)).await;

        // Run the check
        let result = checker.run_check("test_check").await;
        assert!(result.is_some());
        assert!(result.unwrap().status.is_healthy());

        // Check overall health
        let overall = checker.get_overall_health().await;
        assert!(overall.is_healthy());
    }

    #[tokio::test]
    async fn test_health_checker_with_failure() {
        let checker = HealthChecker::new();

        // Add a failing health check
        let check =
            ApplicationHealthCheck::new("failing_check", || Err(anyhow::anyhow!("Always fails")));
        checker.add_check(Box::new(check)).await;

        // Run the check
        let result = checker.run_check("failing_check").await;
        assert!(result.is_some());
        assert!(result.unwrap().status.is_unhealthy());

        // Check overall health
        let overall = checker.get_overall_health().await;
        assert!(overall.is_unhealthy());
    }

    #[tokio::test]
    async fn test_health_checker_should_run() {
        let checker = HealthChecker::with_interval(Duration::from_millis(100));

        // Should run initially
        assert!(checker.should_run_checks().await);

        // Run checks
        checker.run_all_checks().await;

        // Should not run immediately after
        assert!(!checker.should_run_checks().await);

        // Wait for interval to pass
        sleep(Duration::from_millis(150)).await;

        // Should run again
        assert!(checker.should_run_checks().await);
    }
}
