use ultrafast_mcp_core::protocol::jsonrpc::{JsonRpcRequest, JsonRpcResponse};
use ultrafast_mcp_core::protocol::JsonRpcMessage;
use ultrafast_mcp_core::RequestId;
#[cfg(feature = "http")]
use ultrafast_mcp_transport::http::{
    optimization::{OptimizationConfig, PerformanceMonitor, RequestBatcher, ResponseOptimizer},
    server::HttpTransportConfig,
};

#[tokio::test]
#[cfg(feature = "http")]
async fn test_optimization_config_defaults() {
    let config = OptimizationConfig::default();

    assert!(config.enable_request_batching);
    assert_eq!(config.batch_timeout_ms, 10);
    assert_eq!(config.max_batch_size, 100);
    assert!(config.enable_response_streaming);
    assert!(config.enable_connection_multiplexing);
    assert_eq!(config.memory_pool_size, 1024 * 1024);
}

#[tokio::test]
#[cfg(feature = "http")]
async fn test_request_batcher() {
    let config = OptimizationConfig {
        max_batch_size: 3,
        batch_timeout_ms: 50,
        ..Default::default()
    };

    let batcher = RequestBatcher::new(config);

    // Add messages to the same session
    let session_id = "test-session".to_string();
    let message1 = JsonRpcMessage::Request(JsonRpcRequest::new(
        "test_method".to_string(),
        None,
        Some(RequestId::String("1".to_string())),
    ));

    let message2 = JsonRpcMessage::Request(JsonRpcRequest::new(
        "test_method".to_string(),
        None,
        Some(RequestId::String("2".to_string())),
    ));

    let message3 = JsonRpcMessage::Request(JsonRpcRequest::new(
        "test_method".to_string(),
        None,
        Some(RequestId::String("3".to_string())),
    ));

    // Add messages - the third one should trigger batch processing
    batcher
        .add_message(session_id.clone(), message1)
        .await
        .unwrap();
    batcher
        .add_message(session_id.clone(), message2)
        .await
        .unwrap();
    batcher
        .add_message(session_id.clone(), message3)
        .await
        .unwrap();

    // Wait a bit for batch processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[tokio::test]
#[cfg(feature = "http")]
async fn test_response_optimizer() {
    let config = OptimizationConfig::default();
    let optimizer = ResponseOptimizer::new(config);

    // Test small response (should be immediate)
    let small_response = JsonRpcMessage::Response(JsonRpcResponse::success(
        serde_json::json!({"result": "small"}),
        Some(RequestId::String("1".to_string())),
    ));

    let optimized = optimizer.optimize_response(small_response);
    match optimized {
        ultrafast_mcp_transport::http::optimization::OptimizedResponse::Immediate(_) => {
            // Expected for small response
        }
        _ => panic!("Expected immediate response for small payload"),
    }

    // Test large response (should be streamable)
    let large_data = serde_json::json!({
        "result": "x".repeat(1024 * 1024 + 1) // > 1MB
    });

    let large_response = JsonRpcMessage::Response(JsonRpcResponse::success(
        large_data,
        Some(RequestId::String("2".to_string())),
    ));

    let optimized = optimizer.optimize_response(large_response);
    match optimized {
        ultrafast_mcp_transport::http::optimization::OptimizedResponse::Streamable(_) => {
            // Expected for large response
        }
        _ => panic!("Expected streamable response for large payload"),
    }
}

#[tokio::test]
#[cfg(feature = "http")]
async fn test_performance_monitor() {
    let monitor = PerformanceMonitor::new();

    // Record some test metrics
    monitor.record_request(true, 5, 10.5).await;
    monitor.record_request(false, 1, 5.2).await;
    monitor.record_request(true, 3, 8.1).await;

    let metrics = monitor.get_metrics().await;
    println!("average_batch_size: {}", metrics.average_batch_size);

    assert_eq!(metrics.total_requests, 3);
    assert_eq!(metrics.batched_requests, 2);
    assert!((metrics.average_batch_size - 4.0).abs() < 0.1); // (5 + 3) / 2 = 4
    assert!((metrics.average_response_time_ms - 7.93).abs() < 0.1); // (10.5 + 5.2 + 8.1) / 3 ≈ 7.93
}

#[tokio::test]
#[cfg(feature = "http")]
async fn test_http_transport_config_with_optimizations() {
    let config = HttpTransportConfig {
        enable_compression: true,
        enable_caching: true,
        compression_level: 9,   // Maximum compression
        cache_ttl_seconds: 600, // 10 minutes
        optimization_config: OptimizationConfig {
            enable_request_batching: true,
            batch_timeout_ms: 5, // Very fast batching
            max_batch_size: 50,
            enable_response_streaming: true,
            enable_connection_multiplexing: true,
            memory_pool_size: 2 * 1024 * 1024, // 2MB pool
        },
        ..Default::default()
    };

    assert!(config.enable_compression);
    assert!(config.enable_caching);
    assert_eq!(config.compression_level, 9);
    assert_eq!(config.cache_ttl_seconds, 600);
    assert!(config.optimization_config.enable_request_batching);
    assert_eq!(config.optimization_config.batch_timeout_ms, 5);
    assert_eq!(config.optimization_config.max_batch_size, 50);
    assert_eq!(config.optimization_config.memory_pool_size, 2 * 1024 * 1024);
}

#[tokio::test]
#[cfg(feature = "http")]
async fn test_memory_pool() {
    use ultrafast_mcp_transport::http::optimization::MemoryPool;

    let pool = MemoryPool::new(10); // Small pool for testing

    // Get buffers
    let buffer1 = pool.get_buffer(1024).await;
    let buffer2 = pool.get_buffer(512).await;

    assert_eq!(buffer1.len(), 1024);
    assert_eq!(buffer2.len(), 512);

    // Return buffers to pool
    pool.return_buffer(buffer1).await;
    pool.return_buffer(buffer2).await;

    // Get buffer again - should reuse from pool
    let buffer3 = pool.get_buffer(1024).await;
    assert_eq!(buffer3.len(), 1024);
}

#[tokio::test]
#[cfg(feature = "http")]
async fn test_optimization_integration() {
    // Test that all optimization components work together
    let config = OptimizationConfig::default();
    let batcher = RequestBatcher::new(config.clone());
    let optimizer = ResponseOptimizer::new(config.clone());
    let monitor = PerformanceMonitor::new();

    // Simulate a request flow
    let session_id = "integration-test".to_string();
    let request = JsonRpcMessage::Request(JsonRpcRequest::new(
        "test_method".to_string(),
        None,
        Some(RequestId::String("test".to_string())),
    ));

    // Add to batch
    batcher
        .add_message(session_id.clone(), request)
        .await
        .unwrap();

    // Create response
    let response = JsonRpcMessage::Response(JsonRpcResponse::success(
        serde_json::json!({"result": "success"}),
        Some(RequestId::String("test".to_string())),
    ));

    // Optimize response
    let optimized = optimizer.optimize_response(response);
    match optimized {
        ultrafast_mcp_transport::http::optimization::OptimizedResponse::Immediate(_) => {
            // Expected for small response
        }
        _ => panic!("Expected immediate response"),
    }

    // Record metrics
    monitor.record_request(true, 1, 15.0).await;

    let metrics = monitor.get_metrics().await;
    assert_eq!(metrics.total_requests, 1);
    assert_eq!(metrics.batched_requests, 1);
}

#[tokio::test]
#[cfg(feature = "http")]
async fn test_session_resumption_and_message_redelivery() {
    use std::time::Duration;
    use ultrafast_mcp_core::protocol::jsonrpc::JsonRpcRequest;
    use ultrafast_mcp_core::protocol::JsonRpcMessage;
    use ultrafast_mcp_core::RequestId;
    use ultrafast_mcp_transport::http::session::{MessageQueue, SessionStore};

    let session_store = SessionStore::new(2); // 2 seconds timeout
    let message_queue = MessageQueue::new(3); // 3 max retries

    // Create a session
    let session_id = "test-session-resume".to_string();
    let session = session_store.create_session(session_id.clone()).await;
    assert_eq!(session.session_id, session_id);

    // Enqueue a message
    let message = JsonRpcMessage::Request(JsonRpcRequest::new(
        "resume_test".to_string(),
        None,
        Some(RequestId::String("1".to_string())),
    ));
    message_queue
        .enqueue_message(session_id.clone(), message.clone())
        .await;

    // Simulate disconnect (no activity)
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Resume session before timeout
    let resumed = session_store.get_session(&session_id).await;
    assert!(resumed.is_some());

    // Get pending messages (should include our message)
    let pending = message_queue.get_pending_messages(&session_id).await;
    assert_eq!(pending.len(), 1);
    if let (JsonRpcMessage::Request(req1), JsonRpcMessage::Request(req2)) =
        (&pending[0].message, &message)
    {
        assert_eq!(req1.method, req2.method);
        assert_eq!(req1.id, req2.id);
    } else {
        panic!("Expected JsonRpcMessage::Request");
    }

    // Simulate client acknowledging the message
    let msg_id = pending[0].id.clone();
    message_queue
        .acknowledge_message(&session_id, &msg_id)
        .await;
    let pending_after_ack = message_queue.get_pending_messages(&session_id).await;
    assert!(pending_after_ack.is_empty());

    // Enqueue another message and let session expire
    let message2 = JsonRpcMessage::Request(JsonRpcRequest::new(
        "resume_test2".to_string(),
        None,
        Some(RequestId::String("2".to_string())),
    ));
    message_queue
        .enqueue_message(session_id.clone(), message2.clone())
        .await;
    tokio::time::sleep(Duration::from_secs(3)).await; // Wait for session to expire
    let expired = session_store.get_session(&session_id).await;
    assert!(expired.is_none());

    // Test message redelivery with retry logic
    let session_id2 = "test-session-retry".to_string();
    session_store.create_session(session_id2.clone()).await;
    let message3 = JsonRpcMessage::Request(JsonRpcRequest::new(
        "retry_test".to_string(),
        None,
        Some(RequestId::String("3".to_string())),
    ));
    message_queue
        .enqueue_message(session_id2.clone(), message3.clone())
        .await;
    let pending_retry = message_queue.get_pending_messages(&session_id2).await;
    let msg_id_retry = pending_retry[0].id.clone();
    // Increment retry 3 times (should remove after max_retries)
    assert!(
        message_queue
            .increment_retry(&session_id2, &msg_id_retry)
            .await
    );
    assert!(
        message_queue
            .increment_retry(&session_id2, &msg_id_retry)
            .await
    );
    // Third increment should remove the message
    assert!(
        !message_queue
            .increment_retry(&session_id2, &msg_id_retry)
            .await
    );
    let pending_after_retry = message_queue.get_pending_messages(&session_id2).await;
    assert!(pending_after_retry.is_empty());

    // Test get_messages_since
    let session_id3 = "test-session-since".to_string();
    session_store.create_session(session_id3.clone()).await;
    let msg_a = JsonRpcMessage::Request(JsonRpcRequest::new(
        "since_test_a".to_string(),
        None,
        Some(RequestId::String("a".to_string())),
    ));
    let msg_b = JsonRpcMessage::Request(JsonRpcRequest::new(
        "since_test_b".to_string(),
        None,
        Some(RequestId::String("b".to_string())),
    ));
    message_queue
        .enqueue_message(session_id3.clone(), msg_a.clone())
        .await;
    let t0 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    tokio::time::sleep(Duration::from_millis(10)).await;
    message_queue
        .enqueue_message(session_id3.clone(), msg_b.clone())
        .await;
    let all_msgs = message_queue.get_pending_messages(&session_id3).await;
    assert_eq!(all_msgs.len(), 2);
    let since_msgs = message_queue.get_messages_since(&session_id3, t0).await;
    // Only msg_b should be after t0
    assert_eq!(since_msgs.len(), 1);
    if let (JsonRpcMessage::Request(req1), JsonRpcMessage::Request(req2)) =
        (&since_msgs[0].message, &msg_b)
    {
        assert_eq!(req1.method, req2.method);
        assert_eq!(req1.id, req2.id);
    } else {
        panic!("Expected JsonRpcMessage::Request");
    }
}
