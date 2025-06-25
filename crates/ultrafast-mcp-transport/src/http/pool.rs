//! Connection pooling for HTTP transport

use crate::{TransportError, Result};
use std::sync::Arc;
use tokio::sync::{Semaphore, RwLock};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use reqwest::Client;

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_idle_per_host: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(90),
            max_idle_per_host: 10,
        }
    }
}

/// Connection pool for HTTP clients
pub struct ConnectionPool {
    clients: Arc<RwLock<HashMap<String, PooledClient>>>,
    semaphore: Arc<Semaphore>,
    config: PoolConfig,
}

#[derive(Clone)]
struct PooledClient {
    client: Client,
    #[allow(dead_code)]
    created_at: SystemTime,
    last_used: SystemTime,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            config,
        }
    }
    
    /// Get a client for the given host, creating one if necessary
    pub async fn get_client(&self, host: &str) -> Result<Client> {
        // Acquire permit from semaphore
        let _permit = self.semaphore.acquire().await
            .map_err(|_| TransportError::ConnectionError {
                message: "Connection pool exhausted".to_string()
            })?;
        
        let mut clients = self.clients.write().await;
        let now = SystemTime::now();
        
        // Check if we have a valid client for this host
        if let Some(pooled) = clients.get_mut(host) {
            if !self.is_expired(&pooled, now) {
                pooled.last_used = now;
                return Ok(pooled.client.clone());
            } else {
                // Remove expired client
                clients.remove(host);
            }
        }
        
        // Create new client
        let client = Client::builder()
            .timeout(self.config.connection_timeout)
            .pool_idle_timeout(self.config.idle_timeout)
            .pool_max_idle_per_host(self.config.max_idle_per_host)
            .build()
            .map_err(|e| TransportError::InitializationError {
                message: format!("Failed to create HTTP client: {}", e)
            })?;
        
        let pooled_client = PooledClient {
            client: client.clone(),
            created_at: now,
            last_used: now,
        };
        
        clients.insert(host.to_string(), pooled_client);
        Ok(client)
    }
    
    /// Clean up expired connections
    pub async fn cleanup_expired(&self) {
        let mut clients = self.clients.write().await;
        let now = SystemTime::now();
        clients.retain(|_, pooled| !self.is_expired(pooled, now));
    }
    
    fn is_expired(&self, pooled: &PooledClient, _now: SystemTime) -> bool {
        pooled.last_used
            .elapsed()
            .map(|elapsed| elapsed > self.config.idle_timeout)
            .unwrap_or(true)
    }
}

/// Start a background task to clean up expired connections
pub fn start_pool_cleanup(pool: Arc<ConnectionPool>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            pool.cleanup_expired().await;
        }
    });
}
