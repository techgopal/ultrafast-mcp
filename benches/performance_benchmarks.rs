use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ultrafast_mcp_core::*;
use serde_json::json;

fn benchmark_jsonrpc_serialization(c: &mut Criterion) {
    let request = JsonRpcRequest::new(
        RequestId::String("benchmark".to_string()),
        "tools/call".to_string(),
        Some(json!({
            "name": "test_tool",
            "arguments": {
                "input": "test data for benchmarking",
                "number": 42,
                "nested": {
                    "array": [1, 2, 3, 4, 5],
                    "object": {"key": "value"}
                }
            }
        }))
    );

    c.bench_function("jsonrpc_serialization", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(&black_box(&request)).unwrap();
            black_box(serialized)
        })
    });
}

fn benchmark_jsonrpc_deserialization(c: &mut Criterion) {
    let request = JsonRpcRequest::new(
        RequestId::String("benchmark".to_string()),
        "tools/call".to_string(),
        Some(json!({
            "name": "test_tool",
            "arguments": {
                "input": "test data for benchmarking",
                "number": 42,
                "nested": {
                    "array": [1, 2, 3, 4, 5],
                    "object": {"key": "value"}
                }
            }
        }))
    );
    
    let serialized = serde_json::to_string(&request).unwrap();

    c.bench_function("jsonrpc_deserialization", |b| {
        b.iter(|| {
            let deserialized: JsonRpcRequest = serde_json::from_str(&black_box(&serialized)).unwrap();
            black_box(deserialized)
        })
    });
}

fn benchmark_tool_creation(c: &mut Criterion) {
    c.bench_function("tool_creation", |b| {
        b.iter(|| {
            let tool = Tool {
                name: black_box("benchmark_tool".to_string()),
                description: black_box("A tool for benchmarking".to_string()),
                input_schema: black_box(json!({
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"},
                        "count": {"type": "integer", "minimum": 0}
                    },
                    "required": ["input"]
                })),
                output_schema: Some(black_box(json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "string"},
                        "processed_at": {"type": "string", "format": "date-time"}
                    }
                }))),
            };
            black_box(tool)
        })
    });
}

fn benchmark_resource_creation(c: &mut Criterion) {
    c.bench_function("resource_creation", |b| {
        b.iter(|| {
            let resource = Resource {
                uri: black_box("benchmark://resource/test".to_string()),
                name: black_box("Benchmark Resource".to_string()),
                description: Some(black_box("A resource for benchmarking".to_string())),
                mime_type: Some(black_box("application/json".to_string())),
            };
            black_box(resource)
        })
    });
}

fn benchmark_concurrent_message_processing(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("concurrent_messages", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut handles = vec![];
            
            for i in 0..10 {
                let handle = tokio::spawn(async move {
                    let request = JsonRpcRequest::new(
                        RequestId::String(format!("concurrent_{}", i)),
                        "test_method".to_string(),
                        Some(json!({"data": i}))
                    );
                    
                    let serialized = serde_json::to_string(&request).unwrap();
                    let _deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
                    
                    i
                });
                handles.push(handle);
            }
            
            let results: Vec<_> = futures::future::join_all(handles).await;
            black_box(results)
        })
    });
}

criterion_group!(
    benches,
    benchmark_jsonrpc_serialization,
    benchmark_jsonrpc_deserialization,
    benchmark_tool_creation,
    benchmark_resource_creation,
    benchmark_concurrent_message_processing
);
criterion_main!(benches);
