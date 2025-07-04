//! Advanced Features MCP Server
//!
//! This example demonstrates sophisticated MCP patterns and production-ready features:
//! - Advanced data processing with statistical analysis
//! - Multi-step workflow execution with progress tracking
//! - System monitoring with simulated metrics
//! - Complex resource templates and prompt systems

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use ultrafast_mcp::{
    ListToolsRequest, ListToolsResponse, MCPError, MCPResult, ServerCapabilities, ServerInfo, Tool,
    ToolCall, ToolContent, ToolHandler, ToolResult, ToolsCapability, UltraFastServer,
};

#[derive(Debug, Deserialize)]
struct CalculatorRequest {
    operation: String,
    numbers: Vec<f64>,
}

#[derive(Debug, Serialize)]
struct CalculatorResponse {
    operation: String,
    numbers: Vec<f64>,
    result: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct DataProcessorRequest {
    data: Vec<String>,
    operations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DataProcessorResponse {
    original_data: Vec<String>,
    processed_data: Vec<String>,
    operations_applied: Vec<String>,
    processing_time_ms: u64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct MetricsRequest {
    metric_name: String,
    value: f64,
    tags: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
struct MetricsResponse {
    metric_name: String,
    value: f64,
    tags: std::collections::HashMap<String, String>,
    timestamp: DateTime<Utc>,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetricDataPoint {
    name: String,
    value: f64,
    timestamp: DateTime<Utc>,
    tags: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct MetricsReport {
    total_metrics: usize,
    data_points: Vec<MetricDataPoint>,
    generated_at: DateTime<Utc>,
}

struct AdvancedFeaturesHandler {
    metrics: Arc<tokio::sync::RwLock<Vec<MetricDataPoint>>>,
}

impl AdvancedFeaturesHandler {
    fn new() -> Self {
        Self {
            metrics: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl ToolHandler for AdvancedFeaturesHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        info!("Received tool call: {}", call.name);

        match call.name.as_str() {
            "calculator" => {
                let request: CalculatorRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default())
                        .map_err(|e| MCPError::serialization_error(e.to_string()))?;

                self.handle_calculator(request).await
            }
            "data_processor" => {
                let request: DataProcessorRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default())
                        .map_err(|e| MCPError::serialization_error(e.to_string()))?;

                self.handle_data_processor(request).await
            }
            "record_metric" => {
                let request: MetricsRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default())
                        .map_err(|e| MCPError::serialization_error(e.to_string()))?;

                self.handle_record_metric(request).await
            }
            "get_metrics_report" => self.handle_get_metrics_report().await,
            _ => Err(MCPError::method_not_found(format!(
                "Unknown tool: {}",
                call.name
            ))),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![
                Tool {
                    name: "calculator".to_string(),
                    description: "Perform mathematical calculations".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "operation": {
                                "type": "string",
                                "enum": ["add", "subtract", "multiply", "divide", "average"],
                                "description": "Mathematical operation to perform"
                            },
                            "numbers": {
                                "type": "array",
                                "items": { "type": "number" },
                                "description": "Numbers to operate on"
                            }
                        },
                        "required": ["operation", "numbers"]
                    }),
                    output_schema: None,
                },
                Tool {
                    name: "data_processor".to_string(),
                    description: "Process data with various operations".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "data": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Data to process"
                            },
                            "operations": {
                                "type": "array",
                                "items": { "type": "string" },
                                "enum": ["uppercase", "lowercase", "reverse", "sort", "unique"],
                                "description": "Operations to apply"
                            }
                        },
                        "required": ["data", "operations"]
                    }),
                    output_schema: None,
                },
                Tool {
                    name: "record_metric".to_string(),
                    description: "Record a metric with tags".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "metric_name": {
                                "type": "string",
                                "description": "Name of the metric"
                            },
                            "value": {
                                "type": "number",
                                "description": "Metric value"
                            },
                            "tags": {
                                "type": "object",
                                "description": "Optional tags for the metric"
                            }
                        },
                        "required": ["metric_name", "value"]
                    }),
                    output_schema: None,
                },
                Tool {
                    name: "get_metrics_report".to_string(),
                    description: "Get a report of all recorded metrics".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {}
                    }),
                    output_schema: None,
                },
            ],
            next_cursor: None,
        })
    }
}

impl AdvancedFeaturesHandler {
    async fn handle_calculator(&self, request: CalculatorRequest) -> MCPResult<ToolResult> {
        let valid_operations = ["add", "subtract", "multiply", "divide", "average"];

        if !valid_operations.contains(&request.operation.as_str()) {
            return Err(MCPError::invalid_request(format!(
                "Invalid operation: {}",
                request.operation
            )));
        }

        if request.numbers.is_empty() {
            return Err(MCPError::invalid_request(
                "At least one number is required".to_string(),
            ));
        }

        let result = match request.operation.as_str() {
            "add" => request.numbers.iter().sum(),
            "subtract" => {
                let mut iter = request.numbers.iter();
                let first = *iter.next().unwrap();
                iter.fold(first, |acc, &x| acc - x)
            }
            "multiply" => request.numbers.iter().product(),
            "divide" => {
                let mut iter = request.numbers.iter();
                let first = *iter.next().unwrap();
                iter.fold(first, |acc, &x| acc / x)
            }
            "average" => request.numbers.iter().sum::<f64>() / request.numbers.len() as f64,
            _ => unreachable!(),
        };

        let response = CalculatorResponse {
            operation: request.operation,
            numbers: request.numbers,
            result,
            timestamp: Utc::now(),
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_data_processor(&self, request: DataProcessorRequest) -> MCPResult<ToolResult> {
        let valid_operations = ["uppercase", "lowercase", "reverse", "sort", "unique"];

        let invalid_ops: Vec<&String> = request
            .operations
            .iter()
            .filter(|op| !valid_operations.contains(&op.as_str()))
            .collect();

        if !invalid_ops.is_empty() {
            return Err(MCPError::invalid_request(format!(
                "Invalid operations: {:?}",
                invalid_ops
            )));
        }

        let start_time = std::time::Instant::now();
        let mut processed_data = request.data.clone();

        for operation in &request.operations {
            match operation.as_str() {
                "uppercase" => {
                    processed_data = processed_data.iter().map(|s| s.to_uppercase()).collect();
                }
                "lowercase" => {
                    processed_data = processed_data.iter().map(|s| s.to_lowercase()).collect();
                }
                "reverse" => {
                    processed_data = processed_data
                        .iter()
                        .map(|s| s.chars().rev().collect())
                        .collect();
                }
                "sort" => {
                    processed_data.sort();
                }
                "unique" => {
                    let mut seen = std::collections::HashSet::new();
                    processed_data.retain(|x| seen.insert(x.clone()));
                }
                _ => unreachable!(),
            }
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        let response = DataProcessorResponse {
            original_data: request.data,
            processed_data,
            operations_applied: request.operations,
            processing_time_ms: processing_time,
            timestamp: Utc::now(),
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_record_metric(&self, request: MetricsRequest) -> MCPResult<ToolResult> {
        let tags = request.tags.unwrap_or_default();

        let data_point = MetricDataPoint {
            name: request.metric_name.clone(),
            value: request.value,
            timestamp: Utc::now(),
            tags,
        };

        {
            let mut metrics = self.metrics.write().await;
            metrics.push(data_point.clone());
        }

        let response = MetricsResponse {
            metric_name: request.metric_name,
            value: request.value,
            tags: data_point.tags,
            timestamp: data_point.timestamp,
            status: "recorded".to_string(),
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_get_metrics_report(&self) -> MCPResult<ToolResult> {
        let data_points = {
            let metrics = self.metrics.read().await;
            metrics.clone()
        };

        let response = MetricsReport {
            total_metrics: data_points.len(),
            data_points: data_points.clone(),
            generated_at: Utc::now(),
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Advanced Features MCP Server");

    // Create server capabilities
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    // Create server
    let server = UltraFastServer::new(
        ServerInfo {
            name: "advanced-features-server".to_string(),
            version: "1.0.0".to_string(),
            description: Some(
                "An advanced features server demonstrating UltraFastServer".to_string(),
            ),
            authors: None,
            homepage: None,
            license: None,
            repository: None,
        },
        capabilities,
    )
    .with_tool_handler(Arc::new(AdvancedFeaturesHandler::new()));

    info!("Server created, starting stdio transport");

    // Run the server
    server.run_stdio().await?;

    Ok(())
}
