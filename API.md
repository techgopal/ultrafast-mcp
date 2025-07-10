# UltraFast MCP API Documentation

This document provides comprehensive API documentation for the UltraFast MCP framework, covering all crates and their key components.

## Table of Contents

1. [Core Crate (`ultrafast-mcp-core`)](#core-crate)
2. [Server Crate (`ultrafast-mcp-server`)](#server-crate)
3. [Client Crate (`ultrafast-mcp-client`)](#client-crate)
4. [Transport Crate (`ultrafast-mcp-transport`)](#transport-crate)
5. [Auth Crate (`ultrafast-mcp-auth`)](#auth-crate)
6. [Monitoring Crate (`ultrafast-mcp-monitoring`)](#monitoring-crate)
7. [CLI Crate (`ultrafast-mcp-cli`)](#cli-crate)
8. [Main Crate (`ultrafast-mcp`)](#main-crate)

## Core Crate

The `ultrafast-mcp-core` crate provides the foundational types, protocol implementation, and utilities for the MCP ecosystem.

### Key Modules

#### Error Handling

```rust
use ultrafast_mcp_core::error::{MCPError, MCPResult};

// Main error types
pub enum MCPError {
    Protocol(ProtocolError),
    Transport(TransportError),
    ToolExecution(ToolError),
    Resource(ResourceError),
    Authentication(AuthenticationError),
    Validation(ValidationError),
    RateLimit(RateLimitError),
    Serialization(serde_json::Error),
    Io(std::io::Error),
    Other(anyhow::Error),
}

// Convenience constructors
MCPError::invalid_params("Missing required field".to_string());
MCPError::method_not_found("Unknown method".to_string());
MCPError::not_found("Resource not found".to_string());
MCPError::request_timeout();
MCPError::internal_error("Internal server error".to_string());
```

#### Protocol Types

```rust
use ultrafast_mcp_core::protocol::{
    JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, JsonRpcError,
    InitializeRequest, InitializeResponse, InitializedNotification,
    ShutdownRequest, ImplementationMetadata, RequestMetadata, ResponseMetadata,
    LogLevel, LogMessage, Message, Notification, RequestId
};

// JSON-RPC message types
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcRequest),
}

// Lifecycle messages
pub struct InitializeRequest {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

pub struct InitializeResponse {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}
```

#### Core Types

```rust
use ultrafast_mcp_core::types::{
    // Tool types
    Tool, ToolCall, ToolResult, ToolContent, ListToolsRequest, ListToolsResponse,
    
    // Resource types
    Resource, ResourceContent, ReadResourceRequest, ReadResourceResponse,
    ListResourcesRequest, ListResourcesResponse, ResourceTemplate,
    
    // Prompt types
    Prompt, PromptContent, PromptMessage, GetPromptRequest, GetPromptResponse,
    ListPromptsRequest, ListPromptsResponse, PromptArgument,
    
    // Client/Server types
    ClientInfo, ServerInfo, ClientCapabilities, ServerCapabilities,
    
    // Notification types
    ProgressNotification, LoggingMessageNotification, RateLimitNotification,
    
    // Sampling types
    SamplingRequest, SamplingResponse, SamplingContent, SamplingMessage,
    
    // Completion types
    CompleteRequest, CompleteResponse,
    
    // Elicitation types
    ElicitationRequest, ElicitationResponse,
    
    // Roots types
    Root, ListRootsRequest, ListRootsResponse,
};

// Tool definition
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
    pub annotations: Option<serde_json::Value>,
}

// Tool call
pub struct ToolCall {
    pub name: String,
    pub arguments: Option<serde_json::Value>,
}

// Tool result
pub struct ToolResult {
    pub content: Vec<ToolContent>,
    pub is_error: Option<bool>,
}

// Resource content
pub enum ResourceContent {
    Text { uri: String, text: String },
    Binary { uri: String, data: Vec<u8>, mime_type: String },
}
```

#### Schema Generation

```rust
use ultrafast_mcp_core::schema::{
    generate_schema_for, validate_against_schema, validate_tool_input, validate_tool_output,
    object_schema, array_schema, basic_schema, enum_schema
};

// Generate JSON Schema for a type
let schema = generate_schema_for::<MyStruct>();

// Validate data against schema
let is_valid = validate_against_schema(&data, &schema)?;

// Validate tool input/output
let input_valid = validate_tool_input(&input, &tool.input_schema)?;
let output_valid = validate_tool_output(&output, &tool.output_schema)?;
```

#### Utilities

```rust
use ultrafast_mcp_core::utils::{
    PaginationParams, PaginationInfo, Cursor, Progress, ProgressTracker, Uri
};

// Pagination support
pub struct PaginationParams {
    pub cursor: Option<Cursor>,
    pub limit: Option<u32>,
}

pub struct PaginationInfo {
    pub has_more: bool,
    pub next_cursor: Option<Cursor>,
}

// Progress tracking
pub struct Progress {
    pub status: ProgressStatus,
    pub progress: f64,
    pub total: Option<f64>,
    pub message: Option<String>,
}

// URI utilities
pub struct Uri {
    pub scheme: String,
    pub authority: Option<String>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}
```

## Server Crate

The `ultrafast-mcp-server` crate provides the server implementation with handler traits and lifecycle management.

### Core Server

```rust
use ultrafast_mcp_server::{UltraFastServer, ServerInfo, ServerCapabilities};

pub struct UltraFastServer {
    info: ServerInfo,
    capabilities: ServerCapabilities,
    tool_handler: Option<Arc<dyn ToolHandler>>,
    resource_handler: Option<Arc<dyn ResourceHandler>>,
    prompt_handler: Option<Arc<dyn PromptHandler>>,
    sampling_handler: Option<Arc<dyn SamplingHandler>>,
    completion_handler: Option<Arc<dyn CompletionHandler>>,
    elicitation_handler: Option<Arc<dyn ElicitationHandler>>,
    roots_handler: Option<Arc<dyn RootsHandler>>,
}

impl UltraFastServer {
    pub fn new(info: ServerInfo, capabilities: ServerCapabilities) -> Self;
    
    pub fn with_tool_handler(self, handler: Arc<dyn ToolHandler>) -> Self;
    pub fn with_resource_handler(self, handler: Arc<dyn ResourceHandler>) -> Self;
    pub fn with_prompt_handler(self, handler: Arc<dyn PromptHandler>) -> Self;
    pub fn with_sampling_handler(self, handler: Arc<dyn SamplingHandler>) -> Self;
    pub fn with_completion_handler(self, handler: Arc<dyn CompletionHandler>) -> Self;
    pub fn with_elicitation_handler(self, handler: Arc<dyn ElicitationHandler>) -> Self;
    pub fn with_roots_handler(self, handler: Arc<dyn RootsHandler>) -> Self;
    
    pub async fn run_stdio(self) -> MCPResult<()>;
    pub async fn run_streamable_http(self, host: &str, port: u16) -> MCPResult<()>;
}
```

### Handler Traits

#### Tool Handler

```rust
use ultrafast_mcp_server::handlers::ToolHandler;

#[async_trait::async_trait]
pub trait ToolHandler: Send + Sync {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult>;
    async fn list_tools(&self, request: ListToolsRequest) -> MCPResult<ListToolsResponse>;
}
```

#### Resource Handler

```rust
use ultrafast_mcp_server::handlers::ResourceHandler;

#[async_trait::async_trait]
pub trait ResourceHandler: Send + Sync {
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse>;
    async fn list_resources(&self, request: ListResourcesRequest) -> MCPResult<ListResourcesResponse>;
    async fn list_resource_templates(&self, request: ListResourceTemplatesRequest) -> MCPResult<ListResourceTemplatesResponse>;
}
```

#### Prompt Handler

```rust
use ultrafast_mcp_server::handlers::PromptHandler;

#[async_trait::async_trait]
pub trait PromptHandler: Send + Sync {
    async fn get_prompt(&self, request: GetPromptRequest) -> MCPResult<GetPromptResponse>;
    async fn list_prompts(&self, request: ListPromptsRequest) -> MCPResult<ListPromptsResponse>;
}
```

#### Sampling Handler

```rust
use ultrafast_mcp_server::handlers::SamplingHandler;

#[async_trait::async_trait]
pub trait SamplingHandler: Send + Sync {
    async fn create_message(&self, request: CreateMessageRequest) -> MCPResult<CreateMessageResponse>;
}
```

#### Completion Handler

```rust
use ultrafast_mcp_server::handlers::CompletionHandler;

#[async_trait::async_trait]
pub trait CompletionHandler: Send + Sync {
    async fn complete(&self, request: CompleteRequest) -> MCPResult<CompleteResponse>;
}
```

#### Elicitation Handler

```rust
use ultrafast_mcp_server::handlers::ElicitationHandler;

#[async_trait::async_trait]
pub trait ElicitationHandler: Send + Sync {
    async fn elicit_input(&self, request: ElicitationRequest) -> MCPResult<ElicitationResponse>;
}
```

#### Roots Handler

```rust
use ultrafast_mcp_server::handlers::RootsHandler;

#[async_trait::async_trait]
pub trait RootsHandler: Send + Sync {
    async fn list_roots(&self) -> MCPResult<Vec<Root>>;
}
```

### Context Management

```rust
use ultrafast_mcp_server::context::Context;

pub struct Context {
    pub request_id: RequestId,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub capabilities: ServerCapabilities,
}

impl Context {
    pub async fn log_info(&self, message: &str) -> MCPResult<()>;
    pub async fn log_warn(&self, message: &str) -> MCPResult<()>;
    pub async fn log_error(&self, message: &str) -> MCPResult<()>;
    pub async fn progress(&self, message: &str, progress: f64, total: Option<f64>) -> MCPResult<()>;
    pub fn validate_path(&self, path: &Path) -> MCPResult<()>;
}
```

## Client Crate

The `ultrafast-mcp-client` crate provides the client implementation with connection management and retry logic.

### Core Client

```rust
use ultrafast_mcp_client::{UltraFastClient, ClientInfo, ClientCapabilities};

pub struct UltraFastClient {
    info: ClientInfo,
    capabilities: ClientCapabilities,
    state_manager: Arc<RwLock<ClientStateManager>>,
    transport: Arc<RwLock<Option<Box<dyn Transport>>>>,
    request_timeout: Duration,
    timeout_config: Arc<TimeoutConfig>,
}

impl UltraFastClient {
    pub fn new(info: ClientInfo, capabilities: ClientCapabilities) -> Self;
    pub fn new_with_timeout(info: ClientInfo, capabilities: ClientCapabilities, timeout: Duration) -> Self;
    
    pub fn with_timeout(self, timeout: Duration) -> Self;
    pub fn with_timeout_config(self, config: TimeoutConfig) -> Self;
    pub fn with_elicitation_handler(self, handler: Arc<dyn ClientElicitationHandler>) -> Self;
    
    pub async fn connect(&self, transport: Box<dyn Transport>) -> MCPResult<()>;
    pub async fn connect_stdio(&self) -> MCPResult<()>;
    pub async fn connect_http(&self, url: String, auth_token: Option<String>) -> MCPResult<()>;
    
    pub async fn disconnect(&self) -> MCPResult<()>;
    pub async fn shutdown(&self, reason: Option<String>) -> MCPResult<()>;
    
    pub async fn get_state(&self) -> ClientState;
    pub async fn can_operate(&self) -> bool;
    pub async fn get_server_info(&self) -> Option<ServerInfo>;
    pub async fn get_server_capabilities(&self) -> Option<ServerCapabilities>;
}
```

### Client Operations

```rust
impl UltraFastClient {
    // Tool operations
    pub async fn list_tools(&self, request: ListToolsRequest) -> MCPResult<ListToolsResponse>;
    pub async fn list_tools_default(&self) -> MCPResult<ListToolsResponse>;
    pub async fn call_tool(&self, tool_call: ToolCall) -> MCPResult<ToolResult>;
    
    // Resource operations
    pub async fn list_resources(&self, request: ListResourcesRequest) -> MCPResult<ListResourcesResponse>;
    pub async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse>;
    pub async fn subscribe_resource(&self, uri: String) -> MCPResult<()>;
    
    // Prompt operations
    pub async fn list_prompts(&self, request: ListPromptsRequest) -> MCPResult<ListPromptsResponse>;
    pub async fn get_prompt(&self, request: GetPromptRequest) -> MCPResult<GetPromptResponse>;
    
    // Sampling operations
    pub async fn create_message(&self, request: CreateMessageRequest) -> MCPResult<CreateMessageResponse>;
    
    // Completion operations
    pub async fn complete(&self, request: CompleteRequest) -> MCPResult<CompleteResponse>;
    
    // Elicitation operations
    pub async fn respond_to_elicitation(&self, response: ElicitationResponse) -> MCPResult<()>;
    
    // Roots operations
    pub async fn list_roots(&self) -> MCPResult<Vec<Root>>;
    
    // Utility operations
    pub async fn set_log_level(&self, level: LogLevel) -> MCPResult<()>;
    pub async fn ping(&self, data: Option<Value>) -> MCPResult<PingResponse>;
}
```

### Client State Management

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientState {
    Uninitialized,
    Initializing,
    Initialized,
    Operating,
    ShuttingDown,
    Shutdown,
}

impl ClientState {
    pub fn can_operate(&self) -> bool;
    pub fn is_initialized(&self) -> bool;
    pub fn is_shutting_down(&self) -> bool;
}
```

### Client Elicitation Handler

```rust
use ultrafast_mcp_client::ClientElicitationHandler;

#[async_trait::async_trait]
pub trait ClientElicitationHandler: Send + Sync {
    async fn handle_elicitation_request(
        &self,
        request: ElicitationRequest,
    ) -> MCPResult<ElicitationResponse>;
}
```

## Transport Crate

The `ultrafast-mcp-transport` crate provides transport layer implementations for STDIO and HTTP.

### Transport Trait

```rust
use ultrafast_mcp_transport::Transport;

#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()>;
    async fn receive_message(&mut self) -> Result<JsonRpcMessage>;
    async fn close(&mut self) -> Result<()>;
    
    fn get_state(&self) -> ConnectionState;
    fn get_health(&self) -> TransportHealth;
    fn is_ready(&self) -> bool;
    
    async fn shutdown(&mut self, config: ShutdownConfig) -> Result<()>;
    async fn force_shutdown(&mut self) -> Result<()>;
    async fn reconnect(&mut self) -> Result<()>;
    async fn reset(&mut self) -> Result<()>;
}
```

### Connection State

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    ShuttingDown,
    Failed(String),
}
```

### Transport Health

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportHealth {
    pub state: ConnectionState,
    pub last_activity: Option<SystemTime>,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub connection_duration: Option<Duration>,
    pub error_count: u64,
    pub last_error: Option<String>,
}
```

### Transport Configuration

```rust
pub enum TransportConfig {
    Stdio,
    #[cfg(feature = "http")]
    Streamable {
        base_url: String,
        auth_token: Option<String>,
        session_id: Option<String>,
    },
}

pub async fn create_transport(config: TransportConfig) -> Result<Box<dyn Transport>>;
pub async fn create_recovering_transport(
    config: TransportConfig,
    recovery_config: RecoveryConfig,
) -> Result<Box<dyn Transport>>;
```

### Recovery Configuration

```rust
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub enable_jitter: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            enable_jitter: true,
        }
    }
}
```

### Shutdown Configuration

```rust
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    pub graceful_timeout: Duration,
    pub force_timeout: Duration,
    pub drain_pending_messages: bool,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            graceful_timeout: Duration::from_secs(5),
            force_timeout: Duration::from_secs(10),
            drain_pending_messages: true,
        }
    }
}
```

## Auth Crate

The `ultrafast-mcp-auth` crate provides OAuth 2.1 authentication with PKCE support.

### OAuth Client

```rust
use ultrafast_mcp_auth::{OAuthClient, OAuthConfig, AuthResult};

pub struct OAuthClient {
    config: OAuthConfig,
}

impl OAuthClient {
    pub fn from_config(config: OAuthConfig) -> Self;
    
    pub fn client_id(&self) -> &str;
    pub fn client_secret(&self) -> &str;
    pub fn auth_url(&self) -> &str;
    pub fn token_url(&self) -> &str;
    pub fn redirect_uri(&self) -> &str;
    pub fn scopes(&self) -> &[String];
    
    pub async fn get_authorization_url(&self, state: String) -> AuthResult<String>;
    pub async fn get_authorization_url_with_pkce(&self, state: String, pkce_params: PkceParams) -> AuthResult<String>;
    
    pub async fn exchange_code_for_token(
        &self,
        token_url: &str,
        client_id: &str,
        client_secret: Option<&str>,
        redirect_uri: &str,
        code: &str,
        code_verifier: &str,
    ) -> AuthResult<TokenResponse>;
    
    pub async fn refresh_token(
        &self,
        token_url: &str,
        client_id: &str,
        client_secret: Option<&str>,
        refresh_token: &str,
    ) -> AuthResult<TokenResponse>;
}
```

### OAuth Configuration

```rust
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}
```

### PKCE Utilities

```rust
use ultrafast_mcp_auth::{generate_pkce_params, generate_state, generate_session_id};

pub struct PkceParams {
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

pub fn generate_pkce_params() -> AuthResult<PkceParams>;
pub fn generate_state() -> String;
pub fn generate_session_id() -> String;
```

### Token Response

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}
```

### Token Validation

```rust
use ultrafast_mcp_auth::{TokenValidator, AuthResult};

pub struct TokenValidator {
    secret: String,
}

impl TokenValidator {
    pub fn new(secret: String) -> Self;
    pub async fn validate_token(&self, token: &str) -> AuthResult<TokenClaims>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iss: Option<String>,
    pub aud: Option<String>,
    pub exp: Option<u64>,
    pub iat: Option<u64>,
    pub scope: Option<String>,
}
```

### Error Types

```rust
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("OAuth error: {error} - {description}")]
    OAuthError { error: String, description: String },
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Missing scope: {scope}")]
    MissingScope { scope: String },
    
    #[error("Invalid state parameter")]
    InvalidState,
    
    #[error("PKCE verification failed")]
    PkceVerificationFailed,
    
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
}
```

## Monitoring Crate

The `ultrafast-mcp-monitoring` crate provides comprehensive monitoring and observability features.

### Monitoring System

```rust
use ultrafast_mcp_monitoring::{MonitoringSystem, MonitoringConfig};

pub struct MonitoringSystem {
    pub metrics_collector: Arc<MetricsCollector>,
    pub health_checker: Arc<HealthChecker>,
    pub config: MonitoringConfig,
}

impl MonitoringSystem {
    pub fn new(config: MonitoringConfig) -> Self;
    pub async fn init(config: MonitoringConfig) -> Result<Self>;
    
    pub fn metrics(&self) -> Arc<MetricsCollector>;
    pub fn health(&self) -> Arc<HealthChecker>;
    
    pub async fn init_health_checks(&self) -> Result<()>;
    pub async fn start_http_server(&self, addr: SocketAddr) -> Result<()>;
    pub async fn shutdown(&self) -> Result<()>;
}
```

### Metrics Collection

```rust
use ultrafast_mcp_monitoring::{MetricsCollector, RequestTimer};

pub struct MetricsCollector {
    request_metrics: Arc<RwLock<RequestMetrics>>,
    transport_metrics: Arc<RwLock<TransportMetrics>>,
    system_metrics: Arc<RwLock<SystemMetrics>>,
}

impl MetricsCollector {
    pub async fn record_request(&self, method: &str, duration: Duration, success: bool);
    pub async fn record_transport_send(&self, bytes: usize);
    pub async fn record_transport_receive(&self, bytes: usize);
    pub async fn update_system_metrics(&self, cpu_usage: f64, memory_usage: u64, network_io: f64);
    pub async fn get_metrics(&self) -> MetricsSnapshot;
}

pub struct RequestTimer {
    method: String,
    start_time: Instant,
    metrics: Arc<MetricsCollector>,
}

impl RequestTimer {
    pub fn start(method: &str, metrics: Arc<MetricsCollector>) -> Self;
    pub async fn finish(self, success: bool);
}
```

### Health Checking

```rust
use ultrafast_mcp_monitoring::{HealthChecker, HealthCheck, HealthStatus};

pub struct HealthChecker {
    checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck>>>>,
}

impl HealthChecker {
    pub async fn add_check(&self, check: Box<dyn HealthCheck>);
    pub async fn remove_check(&self, name: &str);
    pub async fn check_all(&self) -> HealthStatus;
    pub async fn check_specific(&self, name: &str) -> Option<HealthCheckResult>;
}

#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthCheckResult;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded(Vec<String>),
    Unhealthy(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub duration: Duration,
    pub timestamp: SystemTime,
}
```

### Configuration

```rust
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub metrics: MetricsConfig,
    pub health: HealthConfig,
    pub tracing: TracingConfig,
}

#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub collection_interval: Duration,
    pub retention_period: Duration,
}

#[derive(Debug, Clone)]
pub struct HealthConfig {
    pub enabled: bool,
    pub check_interval: Duration,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub service_name: String,
    pub service_version: String,
    pub enabled: bool,
    pub sampling_rate: f64,
}
```

## CLI Crate

The `ultrafast-mcp-cli` crate provides command-line tools for development and testing.

### Main Commands

```rust
use ultrafast_mcp_cli::{Cli, Commands};

#[derive(Parser)]
#[command(name = "mcp")]
#[command(about = "UltraFast MCP CLI")]
pub struct Cli {
    #[arg(short, long, global = true)]
    verbose: bool,
    
    #[arg(short, long, global = true)]
    debug: bool,
    
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new MCP project
    Init(InitArgs),
    
    /// Generate project scaffolding
    Generate(GenerateArgs),
    
    /// Run a development server
    Dev(DevArgs),
    
    /// Build the project
    Build(BuildArgs),
    
    /// Test MCP connections
    Test(TestArgs),
    
    /// Validate MCP schemas and configurations
    Validate(ValidateArgs),
    
    /// Show project information
    Info(InfoArgs),
    
    /// Manage server configurations
    Server(ServerArgs),
    
    /// Manage client configurations
    Client(ClientArgs),
    
    /// Generate shell completions
    Completions(CompletionsArgs),
}
```

### Command Arguments

```rust
#[derive(Args)]
pub struct InitArgs {
    #[arg(value_name = "PROJECT_NAME")]
    project_name: String,
    
    #[arg(long, default_value = "server")]
    template: String,
    
    #[arg(long)]
    config: Option<PathBuf>,
    
    #[arg(long)]
    force: bool,
    
    #[arg(long)]
    git: bool,
}

#[derive(Args)]
pub struct DevArgs {
    #[arg(long, default_value = "8080")]
    port: u16,
    
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    
    #[arg(long)]
    watch: bool,
    
    #[arg(long)]
    debug: bool,
}

#[derive(Args)]
pub struct TestArgs {
    #[arg(long)]
    server: Option<String>,
    
    #[arg(long)]
    client: Option<PathBuf>,
    
    #[arg(long, default_value = "30")]
    timeout: u64,
    
    #[arg(long)]
    verbose: bool,
}
```

## Main Crate

The main `ultrafast-mcp` crate provides unified APIs and re-exports from all other crates.

### Prelude Module

```rust
use ultrafast_mcp::prelude::*;

// Re-exports all commonly used types
pub use crate::{
    // Server types
    UltraFastServer, ServerInfo, ServerCapabilities,
    
    // Client types
    UltraFastClient, ClientInfo, ClientCapabilities,
    
    // Tool types
    Tool, ToolCall, ToolResult, ToolHandler,
    
    // Resource types
    Resource, ReadResourceRequest, ReadResourceResponse, ResourceHandler,
    
    // Prompt types
    Prompt, GetPromptRequest, GetPromptResponse, PromptHandler,
    
    // Error types
    MCPError, MCPResult,
    
    // Context
    Context,
};

// Capability types
pub use ultrafast_mcp_core::types::{
    ElicitationCapability, RootsCapability, SamplingCapability
};

// Conditional re-exports
#[cfg(feature = "oauth")]
pub use crate::AuthConfig;

#[cfg(feature = "monitoring")]
pub use crate::MonitoringConfig;
```

### Feature Flags

The main crate supports the following feature flags:

- `http`: Enables HTTP transport support
- `oauth`: Enables OAuth 2.1 authentication
- `monitoring`: Enables monitoring and observability features
- `full`: Enables all features

### Usage Example

```rust
use ultrafast_mcp::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create server
    let server_info = ServerInfo {
        name: "my-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("My MCP server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(MyToolHandler));

    // Run server
    server.run_stdio().await?;

    Ok(())
}
```

## Error Handling

All crates use consistent error handling with the `MCPResult<T>` type alias:

```rust
pub type MCPResult<T> = Result<T, MCPError>;

// Error conversion is automatic for common types
let result: MCPResult<String> = std::fs::read_to_string("file.txt")
    .map_err(|e| MCPError::Io(e))?;
```

## Async/Await Support

All operations are async and use `tokio` for the runtime:

```rust
// All handler methods are async
#[async_trait::async_trait]
impl ToolHandler for MyHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        // Async implementation
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(ToolResult { /* ... */ })
    }
}
```

## Serialization

All types support serialization with `serde`:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MyData {
    field: String,
    value: i32,
}

// Automatic JSON Schema generation
let schema = generate_schema_for::<MyData>();
```

This documentation covers the core APIs of the UltraFast MCP framework. For more detailed examples and usage patterns, see the examples directory and individual crate documentation. 