//! Rate limiting for HTTP transport

use crate::{Result, TransportError};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub window_size: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 200,
            window_size: Duration::from_secs(60),
        }
    }
}

/// Token bucket rate limiter
#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_refill: SystemTime,
    capacity: f64,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            tokens: capacity as f64,
            last_refill: SystemTime::now(),
            capacity: capacity as f64,
            refill_rate: refill_rate as f64,
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = SystemTime::now();
        if let Ok(elapsed) = now.duration_since(self.last_refill) {
            let new_tokens = elapsed.as_secs_f64() * self.refill_rate;
            self.tokens = (self.tokens + new_tokens).min(self.capacity);
            self.last_refill = now;
        }
    }
}

/// Rate limiter for HTTP requests
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if a request is allowed for the given identifier (e.g., IP address, session ID)
    pub async fn check_rate_limit(&self, identifier: &str) -> Result<()> {
        let mut buckets = self.buckets.write().await;

        let bucket = buckets.entry(identifier.to_string()).or_insert_with(|| {
            TokenBucket::new(self.config.burst_size, self.config.requests_per_second)
        });

        if bucket.try_consume(1.0) {
            Ok(())
        } else {
            Err(TransportError::NetworkError {
                message: "Rate limit exceeded".to_string(),
            })
        }
    }

    /// Clean up old buckets
    pub async fn cleanup_expired(&self) {
        let mut buckets = self.buckets.write().await;
        let _now = SystemTime::now();

        buckets.retain(|_, bucket| {
            // Remove buckets that haven't been used for more than the window size
            bucket
                .last_refill
                .elapsed()
                .map(|elapsed| elapsed < self.config.window_size * 2)
                .unwrap_or(false)
        });
    }
}

/// Start a background task to clean up expired rate limiters
pub fn start_rate_limit_cleanup(limiter: Arc<RateLimiter>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
        loop {
            interval.tick().await;
            limiter.cleanup_expired().await;
        }
    });
}
