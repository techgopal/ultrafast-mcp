//! # ULTRAFAST MCP
//!
//! **Primary APIs:**
//! - [`UltraFastServer`]: Create MCP servers with ergonomic, type-safe, and full-featured APIs.
//! - [`UltraFastClient`]: Connect to MCP servers with ergonomic, async/await APIs.
//!
//! > All MCP features are supported through these two types. This is the only recommended way to use this crate.
//!
//! All ergonomic types are available at the crate root.
//!
//! ## Example: Server
//!
//! ```rust
//! use ultrafast_mcp::{UltraFastServer, Context, ServerInfo, ServerCapabilities, ToolsCapability};
//! use serde::{Deserialize, Serialize};
//! use std::sync::Arc;
//!
//! #[derive(Deserialize)]
//! struct GreetRequest { name: String }
//!
//! #[derive(Serialize)]
//! struct GreetResponse { message: String }
//!
//! struct MyToolHandler;
//! use ultrafast_mcp::{ToolHandler, ListToolsRequest, ListToolsResponse, MCPError};
//! use ultrafast_mcp_core::types::tools::{ToolCallRequest, ToolCallResponse};
//! use std::future::Future;
//! use std::pin::Pin;
//! use std::boxed::Box;
//! #[async_trait::async_trait]
//! impl ToolHandler for MyToolHandler {
//!     fn handle_tool_call<'life0, 'async_trait>(
//!         &'life0 self,
//!         _req: ToolCallRequest,
//!     ) -> Pin<Box<dyn Future<Output = Result<ToolCallResponse, MCPError>> + Send + 'async_trait>>
//!     where
//!         'life0: 'async_trait,
//!         Self: 'async_trait,
//!     {
//!         Box::pin(async { todo!() })
//!     }
//!     fn list_tools<'life0, 'async_trait>(
//!         &'life0 self,
//!         _req: ListToolsRequest,
//!     ) -> Pin<Box<dyn Future<Output = Result<ListToolsResponse, MCPError>> + Send + 'async_trait>>
//!     where
//!         'life0: 'async_trait,
//!         Self: 'async_trait,
//!     {
//!         Box::pin(async { todo!() })
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let server = UltraFastServer::new(
//!         ServerInfo {
//!             name: "my-server".to_string(),
//!             version: "1.0.0".to_string(),
//!             description: Some("My MCP server".to_string()),
//!             authors: None,
//!             homepage: None,
//!             license: None,
//!             repository: None,
//!         },
//!         ServerCapabilities {
//!             tools: Some(ToolsCapability { list_changed: Some(true) }),
//!             ..Default::default()
//!         }
//!     )
//!     .with_tool_handler(Arc::new(MyToolHandler));
//!     // server.run_stdio().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Example: Client
//!
//! ```rust
//! use ultrafast_mcp::{UltraFastClient, ClientInfo, ClientCapabilities};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let info = ClientInfo {
//!         name: "my-client".to_string(),
//!         version: "1.0.0".to_string(),
//!         authors: None,
//!         description: None,
//!         homepage: None,
//!         repository: None,
//!         license: None,
//!     };
//!     let capabilities = ClientCapabilities::default();
//!     let mut client = UltraFastClient::new(info, capabilities);
//!     // client.with_transport(...); // Set up transport as needed
//!     // let result = client.call_tool("greet", json!({"name": "Alice"})).await?;
//!     // println!("Server says: {}", result);
//!     Ok(())
//! }

// Re-export core types
pub use ultrafast_mcp_core::{
    error::{MCPError, McpResult},
    protocol::capabilities::{
        ClientCapabilities, ElicitationCapability, LoggingCapability, PromptsCapability,
        ResourcesCapability, RootsCapability, SamplingCapability, ServerCapabilities,
        ToolsCapability,
    },
    types::{
        client::ClientInfo,
        completion::{CompleteRequest, CompleteResponse},
        elicitation::{ElicitationRequest, ElicitationResponse},
        notifications::LogLevel,
        prompts::{GetPromptRequest, GetPromptResponse, Prompt},
        resources::{ReadResourceRequest, ReadResourceResponse, Resource, ResourceTemplate},
        roots::Root,
        sampling::{CreateMessageRequest, CreateMessageResponse},
        server::ServerInfo,
        tools::{
            ListToolsRequest, ListToolsResponse, ResourceReference, Tool, ToolCall, ToolContent,
            ToolResult,
        },
    },
};

// Re-export server types
pub use ultrafast_mcp_server::{
    CompletionHandler, Context, ElicitationHandler, PromptHandler, ResourceHandler,
    ResourceSubscriptionHandler, RootsHandler, SamplingHandler, ToolHandler, UltraFastServer,
};

// Re-export client types
pub use ultrafast_mcp_client::{
    ElicitationHandler as ClientElicitationHandler, ResourceChangeHandler,
    SamplingHandler as ClientSamplingHandler, UltraFastClient,
};

// Re-export transport types
pub use ultrafast_mcp_transport::{Transport, TransportConfig};

// Re-export auth types
pub use ultrafast_mcp_auth::OAuthConfig;

#[cfg(feature = "monitoring")]
pub use ultrafast_mcp_monitoring::{MonitoringConfig, MonitoringSystem};
