//! HTTP performance optimizations for UltraFast MCP
//!
//! This module provides advanced performance optimizations for the HTTP transport:
//! - Request batching and coalescing
//! - Response optimization and streaming
//! - Connection multiplexing
//! - Memory pooling and zero-copy operations

use crate::{Result, TransportError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{mpsc, RwLock};
use ultrafast_mcp_core::protocol::JsonRpcMessage;

/// Performance optimization configuration
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub enable_request_batching: bool,
    pub batch_timeout_ms: u64,
    pub max_batch_size: usize,
    pub enable_response_streaming: bool,
    pub enable_connection_multiplexing: bool,
    pub memory_pool_size: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_request_batching: true,
            batch_timeout_ms: 10, // 10ms batching window
            max_batch_size: 100,
            enable_response_streaming: true,
            enable_connection_multiplexing: true,
            memory_pool_size: 1024 * 1024, // 1MB pool
        }
    }
}

/// Request batcher for coalescing multiple requests
pub struct RequestBatcher {
    config: OptimizationConfig,
    pending_requests: Arc<RwLock<HashMap<String, Vec<JsonRpcMessage>>>>,
    batch_sender: mpsc::Sender<BatchRequest>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct BatchRequest {
    session_id: String,
    messages: Vec<JsonRpcMessage>,
    timestamp: Instant,
}

impl RequestBatcher {
    pub fn new(config: OptimizationConfig) -> Self {
        let (batch_sender, mut batch_receiver) = mpsc::channel(1000);
        let pending_requests = Arc::new(RwLock::new(HashMap::new()));

        // Start batch processing task
        let pending_clone = pending_requests.clone();
        let config_clone = config.clone();
        tokio::spawn(async move {
            let mut batch_timer =
                tokio::time::interval(Duration::from_millis(config_clone.batch_timeout_ms));

            loop {
                tokio::select! {
                    _ = batch_timer.tick() => {
                        // Process timed-out batches
                        Self::process_pending_batches(&pending_clone, &config_clone).await;
                    }
                    Some(batch) = batch_receiver.recv() => {
                        // Process immediate batch
                        Self::process_batch(batch, &pending_clone, &config_clone).await;
                    }
                }
            }
        });

        Self {
            config,
            pending_requests,
            batch_sender,
        }
    }

    /// Add a message to the batch for the given session
    pub async fn add_message(&self, session_id: String, message: JsonRpcMessage) -> Result<()> {
        let mut pending = self.pending_requests.write().await;

        let session_batch = pending.entry(session_id.clone()).or_insert_with(Vec::new);
        session_batch.push(message);

        // Check if batch is ready to process
        if session_batch.len() >= self.config.max_batch_size {
            let messages = std::mem::take(session_batch);
            let batch = BatchRequest {
                session_id,
                messages,
                timestamp: Instant::now(),
            };

            self.batch_sender
                .send(batch)
                .await
                .map_err(|e| TransportError::NetworkError {
                    message: format!("Failed to send batch: {}", e),
                })?;
        }

        Ok(())
    }

    /// Process pending batches for all sessions
    async fn process_pending_batches(
        pending: &Arc<RwLock<HashMap<String, Vec<JsonRpcMessage>>>>,
        config: &OptimizationConfig,
    ) {
        let mut pending_guard = pending.write().await;
        let now = Instant::now();
        let _timeout = Duration::from_millis(config.batch_timeout_ms);

        let mut to_process = Vec::new();

        for (session_id, messages) in pending_guard.iter_mut() {
            if !messages.is_empty() {
                to_process.push((session_id.clone(), std::mem::take(messages)));
            }
        }

        // Process batches
        for (session_id, messages) in to_process {
            if !messages.is_empty() {
                let batch = BatchRequest {
                    session_id,
                    messages,
                    timestamp: now,
                };
                Self::process_batch(batch, pending, config).await;
            }
        }
    }

    /// Process a single batch
    async fn process_batch(
        batch: BatchRequest,
        _pending: &Arc<RwLock<HashMap<String, Vec<JsonRpcMessage>>>>,
        _config: &OptimizationConfig,
    ) {
        // Here we would process the batch of messages
        // For now, we just log the batch processing
        tracing::debug!(
            "Processing batch for session {} with {} messages",
            batch.session_id,
            batch.messages.len()
        );
    }
}

/// Response optimizer for efficient response handling
#[allow(dead_code)]
pub struct ResponseOptimizer {
    config: OptimizationConfig,
}

impl ResponseOptimizer {
    pub fn new(config: OptimizationConfig) -> Self {
        Self { config }
    }

    /// Optimize a response for efficient transmission
    pub fn optimize_response(&self, response: JsonRpcMessage) -> OptimizedResponse {
        match response {
            JsonRpcMessage::Response(resp) => {
                // Optimize response based on size and content
                if self.should_stream_response(&resp) {
                    OptimizedResponse::Streamable(resp)
                } else {
                    OptimizedResponse::Immediate(JsonRpcMessage::Response(resp))
                }
            }
            JsonRpcMessage::Notification(notif) => {
                OptimizedResponse::Immediate(JsonRpcMessage::Notification(notif))
            }
            JsonRpcMessage::Request(req) => {
                OptimizedResponse::Immediate(JsonRpcMessage::Request(req))
            }
        }
    }

    /// Determine if a response should be streamed
    fn should_stream_response(
        &self,
        response: &ultrafast_mcp_core::protocol::jsonrpc::JsonRpcResponse,
    ) -> bool {
        // Stream large responses or responses with streaming content
        if let Some(result) = &response.result {
            let size = serde_json::to_string(result).map(|s| s.len()).unwrap_or(0);
            size > 1024 * 1024 // 1MB threshold
        } else {
            false
        }
    }
}

/// Optimized response types
#[derive(Debug)]
pub enum OptimizedResponse {
    Immediate(JsonRpcMessage),
    Streamable(ultrafast_mcp_core::protocol::jsonrpc::JsonRpcResponse),
}

/// Application-level cache for MCP responses
pub struct ResponseCache {
    cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    ttl: Duration,
    max_size: usize,
}

#[derive(Clone)]
struct CachedResponse {
    data: Value,
    expires_at: SystemTime,
}

impl ResponseCache {
    pub fn new(ttl_seconds: u64, max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
            max_size,
        }
    }

    /// Get a cached response if available and not expired
    pub async fn get(&self, key: &str) -> Option<Value> {
        let mut cache = self.cache.write().await;
        let now = SystemTime::now();

        if let Some(cached) = cache.get(key) {
            if cached.expires_at > now {
                return Some(cached.data.clone());
            } else {
                // Remove expired entry
                cache.remove(key);
            }
        }

        None
    }

    /// Store a response in the cache
    pub async fn set(&self, key: String, data: Value) {
        let mut cache = self.cache.write().await;

        // Evict oldest entries if cache is full
        if cache.len() >= self.max_size {
            let oldest_key = cache.keys().next().cloned();
            if let Some(key) = oldest_key {
                cache.remove(&key);
            }
        }

        let expires_at = SystemTime::now() + self.ttl;
        cache.insert(key, CachedResponse { data, expires_at });
    }

    /// Clear expired entries
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let now = SystemTime::now();
        cache.retain(|_, cached| cached.expires_at > now);
    }
}

/// Memory pool for efficient memory allocation
pub struct MemoryPool {
    pool_size: usize,
    available_chunks: Arc<RwLock<Vec<Vec<u8>>>>,
}

impl MemoryPool {
    pub fn new(pool_size: usize) -> Self {
        Self {
            pool_size,
            available_chunks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get a buffer from the pool
    pub async fn get_buffer(&self, size: usize) -> Vec<u8> {
        let mut chunks = self.available_chunks.write().await;

        // Try to find a suitable chunk
        if let Some(index) = chunks.iter().position(|chunk| chunk.len() >= size) {
            chunks.swap_remove(index)
        } else {
            // Create new buffer
            vec![0; size]
        }
    }

    /// Return a buffer to the pool
    pub async fn return_buffer(&self, mut buffer: Vec<u8>) {
        let mut chunks = self.available_chunks.write().await;

        // Only keep buffers up to pool size
        if chunks.len() < self.pool_size {
            buffer.clear();
            chunks.push(buffer);
        }
    }
}

/// Performance metrics for monitoring optimization effectiveness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub batched_requests: u64,
    pub average_batch_size: f64,
    pub compression_ratio: f64,
    pub cache_hit_ratio: f64,
    pub average_response_time_ms: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            batched_requests: 0,
            average_batch_size: 0.0,
            compression_ratio: 1.0,
            cache_hit_ratio: 0.0,
            average_response_time_ms: 0.0,
        }
    }
}

/// Performance monitor for tracking optimization metrics
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }

    /// Record a request
    pub async fn record_request(&self, batched: bool, batch_size: usize, response_time_ms: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;

        if batched {
            metrics.batched_requests += 1;
            // Update average batch size only for batched requests
            let total_batch_size = metrics.average_batch_size
                * (metrics.batched_requests - 1) as f64
                + batch_size as f64;
            metrics.average_batch_size = total_batch_size / metrics.batched_requests as f64;
        }

        // Update average response time
        let total_time = metrics.average_response_time_ms * (metrics.total_requests - 1) as f64
            + response_time_ms;
        metrics.average_response_time_ms = total_time / metrics.total_requests as f64;
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }
}
