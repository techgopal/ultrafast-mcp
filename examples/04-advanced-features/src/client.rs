//! Advanced Features Client Example
//! 
//! This example demonstrates the new UltraFastClient API by testing tools, resources, and prompts.

use serde::{Deserialize, Serialize};
use ultrafast_mcp::{UltraFastClient, ClientInfo, ClientCapabilities, ToolCall, ToolContent};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct CalculatorRequest {
    operation: String,
    numbers: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CalculatorResponse {
    operation: String,
    numbers: Vec<f64>,
    result: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DataProcessorRequest {
    data: Vec<String>,
    operations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DataProcessorResponse {
    original_data: Vec<String>,
    processed_data: Vec<String>,
    operations_applied: Vec<String>,
    processing_time_ms: u64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MetricsRequest {
    metric_name: String,
    value: f64,
    tags: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MetricsResponse {
    metric_name: String,
    value: f64,
    tags: std::collections::HashMap<String, String>,
    timestamp: chrono::DateTime<chrono::Utc>,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MetricsReport {
    total_metrics: usize,
    data_points: Vec<MetricDataPoint>,
    generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MetricDataPoint {
    name: String,
    value: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
    tags: std::collections::HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting Advanced Features MCP Client");
    
    // Create client info and capabilities
    let client_info = ClientInfo {
        name: "advanced-features-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some("An advanced features client demonstrating UltraFastClient".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };
    
    let client_capabilities = ClientCapabilities {
        ..Default::default()
    };
    
    // Create client
    let client = UltraFastClient::new(client_info, client_capabilities);
    
    info!("Connecting to server via stdio");
    
    // Connect to server
    client.connect_stdio().await?;
    
    info!("Connected! Listing available tools");
    
    // List available tools
    let tools = client.list_tools().await?;
    info!("Available tools: {:?}", tools);
    
    // Test Calculator
    let calc_request = CalculatorRequest {
        operation: "add".to_string(),
        numbers: vec![1.0, 2.0, 3.0, 4.0, 5.0],
    };
    
    let tool_call = ToolCall {
        name: "calculator".to_string(),
        arguments: Some(serde_json::to_value(calc_request)?),
    };
    
    info!("Testing calculator with addition");
    let result = client.call_tool(tool_call).await?;
    
    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Calculator response: {}", text);
                let response: CalculatorResponse = serde_json::from_str(&text)?;
                println!("Calculator: {} of {:?} = {}", response.operation, response.numbers, response.result);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }
    
    // Test Data Processor
    let data_request = DataProcessorRequest {
        data: vec!["hello".to_string(), "world".to_string(), "test".to_string()],
        operations: vec!["uppercase".to_string(), "sort".to_string()],
    };
    
    let tool_call = ToolCall {
        name: "data_processor".to_string(),
        arguments: Some(serde_json::to_value(data_request)?),
    };
    
    info!("Testing data processor");
    let result = client.call_tool(tool_call).await?;
    
    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Data processor response: {}", text);
                let response: DataProcessorResponse = serde_json::from_str(&text)?;
                println!("Data processor: {} operations applied in {}ms", response.operations_applied.len(), response.processing_time_ms);
                println!("Original: {:?}", response.original_data);
                println!("Processed: {:?}", response.processed_data);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }
    
    // Test Metrics Recording
    let mut tags = std::collections::HashMap::new();
    tags.insert("service".to_string(), "advanced-features".to_string());
    tags.insert("version".to_string(), "1.0.0".to_string());
    
    let metric_request = MetricsRequest {
        metric_name: "test_metric".to_string(),
        value: 42.5,
        tags: Some(tags),
    };
    
    let tool_call = ToolCall {
        name: "record_metric".to_string(),
        arguments: Some(serde_json::to_value(metric_request)?),
    };
    
    info!("Testing metric recording");
    let result = client.call_tool(tool_call).await?;
    
    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Metric recording response: {}", text);
                let response: MetricsResponse = serde_json::from_str(&text)?;
                println!("Metric recorded: {} = {} ({})", response.metric_name, response.value, response.status);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }
    
    // Test Metrics Report
    let tool_call = ToolCall {
        name: "get_metrics_report".to_string(),
        arguments: Some(serde_json::json!({})),
    };
    
    info!("Testing metrics report");
    let result = client.call_tool(tool_call).await?;
    
    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Metrics report response: {}", text);
                let response: MetricsReport = serde_json::from_str(&text)?;
                println!("Metrics report: {} total metrics", response.total_metrics);
                for data_point in response.data_points {
                    println!("  - {}: {} at {}", data_point.name, data_point.value, data_point.timestamp);
                }
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }
    
    info!("Disconnecting from server");
    client.disconnect().await?;
    
    Ok(())
} 