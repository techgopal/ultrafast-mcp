//! Performance benchmarks for HTTP transport

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use tokio::runtime::Runtime;
use ultrafast_mcp_transport::{
    http::{HttpTransportServer, HttpTransportConfig, RateLimiter, RateLimitConfig, ConnectionPool, PoolConfig},
};
use ultrafast_mcp_core::protocol::{JsonRpcMessage, Request, Notification};
use std::time::Duration;

fn bench_rate_limiter(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("rate_limiter");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("single_client", |b| {
        let config = RateLimitConfig {
            requests_per_second: 1000,
            burst_size: 1000,
            window_size: Duration::from_secs(60),
        };
        let limiter = RateLimiter::new(config);
        
        b.to_async(&rt).iter(|| async {
            black_box(limiter.check_rate_limit("client1").await.unwrap())
        });
    });
    
    group.bench_function("multiple_clients", |b| {
        let config = RateLimitConfig {
            requests_per_second: 1000,
            burst_size: 1000,
            window_size: Duration::from_secs(60),
        };
        let limiter = RateLimiter::new(config);
        
        b.to_async(&rt).iter(|| async {
            for i in 0..10 {
                let client_id = format!("client{}", i);
                black_box(limiter.check_rate_limit(&client_id).await.unwrap());
            }
        });
    });
    
    group.finish();
}

fn bench_connection_pool(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("connection_pool");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("get_client", |b| {
        let config = PoolConfig::default();
        let pool = ConnectionPool::new(config);
        
        b.to_async(&rt).iter(|| async {
            black_box(pool.get_client("example.com").await.unwrap())
        });
    });
    
    group.bench_function("get_multiple_hosts", |b| {
        let config = PoolConfig::default();
        let pool = ConnectionPool::new(config);
        
        b.to_async(&rt).iter(|| async {
            for i in 0..5 {
                let host = format!("example{}.com", i);
                black_box(pool.get_client(&host).await.unwrap());
            }
        });
    });
    
    group.finish();
}

fn bench_session_management(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("session_management");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("create_session", |b| {
        let config = HttpTransportConfig::default();
        let server = HttpTransportServer::new(config);
        let session_store = server.get_state().session_store;
        let mut counter = 0;
        
        b.to_async(&rt).iter(|| async {
            counter += 1;
            let session_id = format!("session_{}", counter);
            black_box(session_store.create_session(session_id).await)
        });
    });
    
    group.bench_function("get_session", |b| {
        let config = HttpTransportConfig::default();
        let server = HttpTransportServer::new(config);
        let session_store = server.get_state().session_store;
        
        // Pre-create a session
        rt.block_on(async {
            session_store.create_session("test_session".to_string()).await;
        });
        
        b.to_async(&rt).iter(|| async {
            black_box(session_store.get_session("test_session").await.unwrap())
        });
    });
    
    group.finish();
}

fn bench_message_queue(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("message_queue");
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("enqueue_message", |b| {
        let config = HttpTransportConfig::default();
        let server = HttpTransportServer::new(config);
        let message_queue = server.get_state().message_queue;
        
        let test_message = JsonRpcMessage::Notification(Notification {
            method: "test/notification".to_string(),
            params: None,
        });
        
        b.to_async(&rt).iter(|| async {
            black_box(message_queue.enqueue_message("test_session".to_string(), test_message.clone()).await)
        });
    });
    
    group.bench_function("get_pending_messages", |b| {
        let config = HttpTransportConfig::default();
        let server = HttpTransportServer::new(config);
        let message_queue = server.get_state().message_queue;
        
        // Pre-enqueue some messages
        rt.block_on(async {
            let test_message = JsonRpcMessage::Notification(Notification {
                method: "test/notification".to_string(),
                params: None,
            });
            
            for _ in 0..10 {
                message_queue.enqueue_message("test_session".to_string(), test_message.clone()).await;
            }
        });
        
        b.to_async(&rt).iter(|| async {
            black_box(message_queue.get_pending_messages("test_session").await)
        });
    });
    
    group.finish();
}

fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_operations");
    group.throughput(Throughput::Elements(10));
    
    group.bench_function("concurrent_rate_limiting", |b| {
        let config = RateLimitConfig {
            requests_per_second: 10000,
            burst_size: 10000,
            window_size: Duration::from_secs(60),
        };
        let limiter = RateLimiter::new(config);
        
        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();
            for i in 0..10 {
                let limiter = limiter.clone();
                let client_id = format!("client{}", i);
                handles.push(tokio::spawn(async move {
                    limiter.check_rate_limit(&client_id).await.unwrap()
                }));
            }
            
            for handle in handles {
                black_box(handle.await.unwrap());
            }
        });
    });
    
    group.bench_function("concurrent_session_access", |b| {
        let config = HttpTransportConfig::default();
        let server = HttpTransportServer::new(config);
        let session_store = server.get_state().session_store;
        
        // Pre-create sessions
        rt.block_on(async {
            for i in 0..10 {
                session_store.create_session(format!("session_{}", i)).await;
            }
        });
        
        b.to_async(&rt).iter(|| async {
            let mut handles = Vec::new();
            for i in 0..10 {
                let session_store = session_store.clone();
                let session_id = format!("session_{}", i);
                handles.push(tokio::spawn(async move {
                    session_store.get_session(&session_id).await.unwrap()
                }));
            }
            
            for handle in handles {
                black_box(handle.await.unwrap());
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    http_transport_benches,
    bench_rate_limiter,
    bench_connection_pool,
    bench_session_management,
    bench_message_queue,
    bench_concurrent_operations
);
criterion_main!(http_transport_benches);
