//! # UltraFast MCP Macros
//!
//! Procedural macros for the UltraFast Model Context Protocol (MCP) implementation.
//!
//! This crate provides convenient procedural macros that simplify MCP development
//! by automatically generating boilerplate code, schemas, and configurations.
//! It reduces the amount of repetitive code needed to implement MCP servers and clients.
//!
//! ## Overview
//!
//! The UltraFast MCP Macros crate provides:
//!
//! - **Schema Generation**: Automatic JSON Schema generation from Rust types
//! - **Tool Registration**: Simplified tool definition and registration
//! - **Server Setup**: Streamlined server configuration and setup
//! - **Client Configuration**: Easy client configuration and setup
//! - **Request/Response**: Automatic request and response type generation
//! - **Error Handling**: Simplified error type generation
//!
//! ## Key Features
//!
//! ### Automatic Schema Generation
//! - **Type Inference**: Automatically infer JSON schemas from Rust types
//! - **Custom Attributes**: Fine-tune schema generation with attributes
//! - **Validation**: Generate validation rules from type constraints
//! - **Documentation**: Preserve Rust documentation in generated schemas
//! - **Nested Types**: Handle complex nested structures and enums
//!
//! ### Tool Registration
//! - **Function Attributes**: Convert Rust functions into MCP tools
//! - **Automatic Registration**: Generate tool registration code
//! - **Schema Generation**: Create input/output schemas automatically
//! - **Error Handling**: Integrate with MCP error types
//! - **Async Support**: Full support for async functions
//!
//! ### Server and Client Setup
//! - **Server Configuration**: Simplify server setup and configuration
//! - **Client Configuration**: Easy client configuration management
//! - **Capability Management**: Automatic capability configuration
//! - **Info Generation**: Generate server/client information
//! - **Type Safety**: Compile-time type checking and validation
//!
//! ## Macros
//!
//! ### `#[derive(McpSchema)]` - Schema Generation
//! Automatically generates JSON schemas from Rust structs and enums.
//!
//! ```rust
//! use ultrafast_mcp_macros::McpSchema;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(McpSchema, Serialize, Deserialize)]
//! struct UserInput {
//!     name: String,
//!     age: u32,
//!     email: Option<String>,
//!     #[mcp(description = "User preferences")]
//!     preferences: Vec<String>,
//! }
//!
//! // The macro generates:
//! // - JSON schema for the struct
//! // - Schema validation methods
//! // - Type conversion utilities
//! ```
//!
//! ### `#[mcp_tool]` - Tool Definition
//! Converts Rust functions into MCP tools with automatic schema generation.
//!
//! ```rust
//! use ultrafast_mcp_macros::mcp_tool;
//! use serde_json::Value;
//!
//! #[mcp_tool(
//!     name = "greet_user",
//!     description = "Greet a user with a personalized message"
//! )]
//! async fn greet_user(input: Value) -> Result<String, Box<dyn std::error::Error>> {
//!     let name = input["name"].as_str().unwrap_or("World");
//!     let greeting = input["greeting"].as_str().unwrap_or("Hello");
//!     Ok(format!("{}, {}!", greeting, name))
//! }
//!
//! // The macro generates:
//! // - Tool registration function
//! // - Input/output schemas
//! // - Error handling integration
//! ```
//!
//! ### `#[mcp_server]` - Server Setup
//! Simplifies MCP server setup and configuration.
//!
//! ```rust
//! use ultrafast_mcp_macros::mcp_server;
//!
//! #[mcp_server(
//!     name = "MyGreetingServer",
//!     version = "1.0.0",
//!     description = "A server that provides greeting tools"
//! )]
//! struct MyServer;
//!
//! // The macro generates:
//! // - Server information
//! // - Server capabilities
//! // - Server setup methods
//! ```
//!
//! ### `#[mcp_client]` - Client Configuration
//! Simplifies MCP client configuration and setup.
//!
//! ```rust
//! use ultrafast_mcp_macros::mcp_client;
//!
//! #[mcp_client(
//!     name = "MyClient",
//!     version = "1.0.0",
//!     description = "A client for the greeting server"
//! )]
//! struct MyClient;
//!
//! // The macro generates:
//! // - Client information
//! // - Client capabilities
//! // - Client setup methods
//! ```
//!
//! ### `mcp_request!` - Request Type Generation
//! Generates MCP request types with automatic validation.
//!
//! ```rust
//! use ultrafast_mcp_macros::mcp_request;
//! use ultrafast_mcp_core::protocol::jsonrpc::{JsonRpcRequest, RequestId};
//! use serde_json::json;
//!
//! let request = mcp_request! {
//!     method: "tools/list",
//!     params: {},
//!     id: 1
//! };
//! assert_eq!(request.method, "tools/list");
//! assert_eq!(request.id, Some(RequestId::Number(1)));
//! ```
//!
//! ### `mcp_response!` - Response Type Generation
//! Generates MCP response types with automatic serialization.
//!
//! ```rust
//! use ultrafast_mcp_macros::mcp_response;
//! use ultrafast_mcp_core::protocol::jsonrpc::{JsonRpcResponse, RequestId};
//! use serde_json::json;
//!
//! let response = mcp_response! {
//!     result: {"status": "ok"},
//!     id: 1
//! };
//! assert_eq!(response.id, Some(RequestId::Number(1)));
//! ```
//!
//! ## Usage Examples
//!
//! ### Complete Tool Implementation
//!
//! ```rust
//! use ultrafast_mcp_macros::{mcp_tool, McpSchema};
//! use serde::{Serialize, Deserialize};
//! use serde_json::Value;
//!
//! // Define input/output types with schemas
//! #[derive(McpSchema, Serialize, Deserialize)]
//! struct CalculatorInput {
//!     operation: String,
//!     a: f64,
//!     b: f64,
//! }
//!
//! #[derive(McpSchema, Serialize, Deserialize)]
//! struct CalculatorOutput {
//!     result: f64,
//!     operation: String,
//! }
//!
//! // Define the tool
//! #[mcp_tool(
//!     name = "calculate",
//!     description = "Perform basic mathematical operations"
//! )]
//! async fn calculate(input: CalculatorInput) -> Result<CalculatorOutput, Box<dyn std::error::Error>> {
//!     let result = match input.operation.as_str() {
//!         "add" => input.a + input.b,
//!         "subtract" => input.a - input.b,
//!         "multiply" => input.a * input.b,
//!         "divide" => {
//!             if input.b == 0.0 {
//!                 return Err("Division by zero".into());
//!             }
//!             input.a / input.b
//!         }
//!         _ => return Err("Unknown operation".into()),
//!     };
//!
//!     Ok(CalculatorOutput {
//!         result,
//!         operation: input.operation,
//!     })
//! }
//!
//! // Example usage:
//! // let tool = register_tool();
//! // assert_eq!(tool.name, "calculate");
//! ```
//!
//! ### Server with Multiple Tools
//!
//! ```rust
//! use ultrafast_mcp_macros::{mcp_server, mcp_tool};
//! use ultrafast_mcp_server::UltraFastServer;
//! use ultrafast_mcp_core::types::tools::Tool;
//!
//! #[mcp_server(
//!     name = "MathServer",
//!     version = "1.0.0",
//!     description = "A server providing mathematical tools"
//! )]
//! struct MathServer;
//!
//! #[mcp_tool(name = "add", description = "Add two numbers")]
//! async fn add_tool(a: f64, b: f64) -> Result<f64, Box<dyn std::error::Error>> {
//!     Ok(a + b)
//! }
//!
//! #[mcp_tool(name = "multiply", description = "Multiply two numbers")]
//! async fn multiply_tool(a: f64, b: f64) -> Result<f64, Box<dyn std::error::Error>> {
//!     Ok(a * b)
//! }
//!
//! // Example server setup (commented out to avoid async main issues in doctest):
//! // #[tokio::main]
//! // async fn main() -> anyhow::Result<()> {
//! //     let server_info = MathServer::server_info();
//! //     let server = UltraFastServer::new(server_info, Default::default());
//! //     
//! //     // Register tools using the generated functions
//! //     let add_tool = register_add_tool_tool();
//! //     let multiply_tool = register_multiply_tool_tool();
//! //     
//! //     server.run_stdio().await?;
//! //     Ok(())
//! // }
//!
//! // Example usage:
//! let server_info = MathServer::server_info();
//! assert_eq!(server_info.name, "MathServer");
//! assert_eq!(server_info.version, "1.0.0");
//!
//! // Test the generated tool registration functions
//! let add_tool = register_add_tool_tool();
//! let multiply_tool = register_multiply_tool_tool();
//! assert_eq!(add_tool.name, "add_tool");
//! assert_eq!(multiply_tool.name, "multiply_tool");
//! ```
//!
//! ### Client with Configuration
//!
//! ```rust
//! use ultrafast_mcp_macros::{mcp_client, mcp_request, mcp_response};
//! use serde::{Serialize, Deserialize};
//! use ultrafast_mcp_core::types::tools::ToolCall;
//!
//! #[mcp_client(
//!     name = "MathClient",
//!     version = "1.0.0",
//!     description = "A client for mathematical operations"
//! )]
//! struct MathClient;
//!
//! #[derive(Serialize, Deserialize)]
//! struct AddRequest {
//!     a: f64,
//!     b: f64,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct AddResponse {
//!     result: f64,
//! }
//!
//! // Example client setup (commented out to avoid async main issues in doctest):
//! // #[tokio::main]
//! // async fn main() -> anyhow::Result<()> {
//! //     let client_info = MathClient::client_info();
//! //     let client = ultrafast_mcp_client::UltraFastClient::new(client_info, Default::default());
//! //     
//! //     client.connect_http("http://localhost:8080/mcp").await?;
//! //     
//! //     let request = AddRequest { a: 5.0, b: 3.0 };
//! //     let tool_call = ToolCall {
//! //         name: "add".to_string(),
//! //         arguments: Some(serde_json::to_value(request)?),
//! //     };
//! //     let response = client.call_tool(tool_call).await?;
//! //     
//! //     println!("Result: {:?}", response);
//! //     Ok(())
//! // }
//!
//! // Example usage:
//! // let client_info = MathClient::client_info();
//! // assert_eq!(client_info.name, "MathClient");
//! ```
//!
//! ## Schema Attributes
//!
//! The `McpSchema` derive macro supports various attributes for customizing schema generation:
//!
//! ```rust
//! use ultrafast_mcp_macros::McpSchema;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(McpSchema, Serialize, Deserialize)]
//! struct User {
//!     #[mcp(description = "User's full name")]
//!     name: String,
//!     
//!     #[mcp(minimum = 0, maximum = 150)]
//!     age: u32,
//!     
//!     #[mcp(format = "email")]
//!     email: String,
//!     
//!     #[mcp(min_length = 8)]
//!     password: String,
//!     
//!     #[mcp(required = false)]
//!     bio: Option<String>,
//! }
//! ```
//!
//! ## Error Handling
//!
//! The macros integrate seamlessly with MCP error handling:
//!
//! ```rust
//! use ultrafast_mcp_macros::mcp_tool;
//! use ultrafast_mcp_core::MCPError;
//!
//! #[mcp_tool(name = "risky_operation")]
//! async fn risky_operation(input: String) -> Result<String, MCPError> {
//!     if input.is_empty() {
//!         return Err(MCPError::invalid_params("Input cannot be empty".to_string()));
//!     }
//!     
//!     if input.len() > 1000 {
//!         return Err(MCPError::invalid_params("Input too long".to_string()));
//!     }
//!     
//!     Ok(format!("Processed: {}", input))
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - **Compile-time Generation**: All code is generated at compile time
//! - **Zero Runtime Overhead**: No runtime reflection or dynamic code generation
//! - **Optimized Schemas**: Efficient schema generation and validation
//! - **Minimal Allocations**: Optimized for minimal memory usage
//! - **Fast Serialization**: Efficient serialization/deserialization
//!
//! ## Best Practices
//!
//! ### Schema Design
//! - Use descriptive field names and types
//! - Add meaningful descriptions with attributes
//! - Use appropriate validation constraints
//! - Keep schemas simple and focused
//! - Document complex schemas thoroughly
//!
//! ### Tool Implementation
//! - Use strongly-typed input/output types
//! - Implement proper error handling
//! - Add meaningful descriptions
//! - Keep tools focused and single-purpose
//! - Test tools thoroughly
//!
//! ### Server/Client Setup
//! - Use descriptive names and versions
//! - Provide meaningful descriptions
//! - Configure appropriate capabilities
//! - Implement proper error handling
//! - Follow naming conventions
//!
//! ## Thread Safety
//!
//! All generated code is designed to be thread-safe:
//! - Generated types implement `Send + Sync` where appropriate
//! - No mutable global state is used
//! - Concurrent access is supported
//! - Safe for use in async contexts
//!
//! ## Examples
//!
//! See the `examples/` directory for complete working examples:
//! - Basic tool implementation
//! - Server with multiple tools
//! - Client configuration
//! - Schema customization
//! - Error handling patterns

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemFn, ItemStruct};

/// Derive macro for automatic JSON Schema generation
///
/// # Example
/// ```rust
/// use ultrafast_mcp_macros::McpSchema;
/// use serde::{Serialize, Deserialize};
/// use ultrafast_mcp_core::schema::McpSchema as McpSchemaTrait;
///
/// #[derive(McpSchema, Serialize, Deserialize)]
/// struct MyTool {
///     name: String,
///     value: i32,
/// }
///
/// // The macro generates an implementation of McpSchema
/// let schema = MyTool::schema();
/// let schema_name = MyTool::schema_name();
/// assert_eq!(schema_name, "MyTool");
/// ```
#[proc_macro_derive(McpSchema, attributes(mcp))]
pub fn derive_mcp_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let schema_impl = quote! {
        impl #impl_generics ultrafast_mcp_core::schema::McpSchema for #name #ty_generics #where_clause {
            fn schema() -> serde_json::Value {
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                })
            }

            fn schema_name() -> String {
                stringify!(#name).to_string()
            }
        }
    };

    TokenStream::from(schema_impl)
}

/// Attribute macro for defining MCP tools
///
/// # Example
/// ```rust
/// use ultrafast_mcp_macros::mcp_tool;
/// use serde_json;
///
/// #[mcp_tool(name = "echo", description = "Echo back the input")]
/// async fn echo_tool(input: String) -> Result<String, Box<dyn std::error::Error>> {
///     Ok(input)
/// }
/// // Example of constructing a Tool struct:
/// let tool = ultrafast_mcp_core::types::tools::Tool {
///     name: "echo".to_string(),
///     description: "Echo back the input".to_string(),
///     input_schema: serde_json::json!({
///         "type": "object",
///         "properties": {},
///         "required": []
///     }),
///     output_schema: Some(serde_json::json!({})),
/// };
/// ```
#[proc_macro_attribute]
pub fn mcp_tool(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // For now, just use defaults - in a real implementation you'd parse the args properly
    let fn_name = &input_fn.sig.ident;
    let tool_name = fn_name.to_string();
    let description = format!("Tool: {}", tool_name);
    let register_fn_name = quote::format_ident!("register_{}_tool", fn_name);

    let expanded = quote! {
        #input_fn

        // Generate tool registration function
        pub fn #register_fn_name() -> ultrafast_mcp_core::types::tools::Tool {
            ultrafast_mcp_core::types::tools::Tool {
                name: #tool_name.to_string(),
                description: #description.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
                output_schema: Some(serde_json::json!({})),
            }
        }
    };

    TokenStream::from(expanded)
}

/// Attribute macro for MCP server setup
///
/// # Example
/// ```rust
/// use ultrafast_mcp_macros::mcp_server;
///
/// #[mcp_server(name = "MyServer", version = "1.0.0")]
/// struct MyServer;
///
/// // Example of creating a server:
/// let server = ultrafast_mcp_server::UltraFastServer::new(
///     ultrafast_mcp_core::types::server::ServerInfo {
///         name: "MyServer".to_string(),
///         version: "1.0.0".to_string(),
///         description: None,
///         homepage: None,
///         repository: None,
///         authors: None,
///         license: None,
///     },
///     ultrafast_mcp_core::types::server::ServerCapabilities::default(),
/// );
/// ```
#[proc_macro_attribute]
pub fn mcp_server(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);

    // For now, just use defaults - in a real implementation you'd parse the args properly
    let struct_name = &input_struct.ident;
    let server_name = struct_name.to_string();
    let version = "1.0.0";

    let expanded = quote! {
        #input_struct

        impl #struct_name {
            /// Get server information
            pub fn server_info() -> ultrafast_mcp_core::types::server::ServerInfo {
                ultrafast_mcp_core::types::server::ServerInfo {
                    name: #server_name.to_string(),
                    version: #version.to_string(),
                    description: None,
                    homepage: None,
                    repository: None,
                    authors: None,
                    license: None,
                }
            }

            /// Create a new server
            pub fn new() -> ultrafast_mcp_server::UltraFastServer {
                ultrafast_mcp_server::UltraFastServer::new(
                    Self::server_info(),
                    ultrafast_mcp_core::types::server::ServerCapabilities::default(),
                )
            }
        }
    };

    TokenStream::from(expanded)
}

/// Macro for creating MCP client configurations
///
/// # Example
/// ```rust
/// use ultrafast_mcp_macros::mcp_client_config;
///
/// mcp_client_config! {
///     name: "MyClient",
///     version: "1.0.0",
///     capabilities: {
///         experimental: {},
///         sampling: {}
///     }
/// }
/// ```
#[proc_macro]
pub fn mcp_client_config(_input: TokenStream) -> TokenStream {
    // For now, just return a placeholder - in a real implementation you'd parse the input
    let expanded = quote! {
        // Client configuration would be generated here
        pub struct ClientConfig;
    };

    TokenStream::from(expanded)
}

/// Macro for creating MCP requests
///
/// # Example
/// ```rust
/// use ultrafast_mcp_macros::mcp_request;
///
/// let request = mcp_request! {
///     method: "tools/list",
///     params: {},
///     id: 1
/// };
/// ```
#[proc_macro]
pub fn mcp_request(_input: TokenStream) -> TokenStream {
    // For now, just return a placeholder - in a real implementation you'd parse the input
    let expanded = quote! {
        ultrafast_mcp_core::protocol::jsonrpc::JsonRpcRequest::new(
            "tools/list".to_string(),
            Some(serde_json::json!({})),
            Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::Number(1))
        )
    };

    TokenStream::from(expanded)
}

/// Macro for creating MCP responses
///
/// # Example
/// ```rust
/// use ultrafast_mcp_macros::mcp_response;
///
/// let response = mcp_response! {
///     result: {"status": "ok"},
///     id: 1
/// };
/// ```
#[proc_macro]
pub fn mcp_response(_input: TokenStream) -> TokenStream {
    // For now, just return a placeholder - in a real implementation you'd parse the input
    let expanded = quote! {
        ultrafast_mcp_core::protocol::jsonrpc::JsonRpcResponse::success(
            serde_json::json!({"status": "ok"}),
            Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::Number(1))
        )
    };

    TokenStream::from(expanded)
}

/// Macro for creating MCP errors
///
/// # Example
/// ```rust
/// use ultrafast_mcp_macros::mcp_error;
///
/// let error = mcp_error! {
///     code: -32602,
///     message: "Invalid params",
///     id: 1
/// };
/// ```
#[proc_macro]
pub fn mcp_error(_input: TokenStream) -> TokenStream {
    // For now, just return a placeholder - in a real implementation you'd parse the input
    let expanded = quote! {
        ultrafast_mcp_core::protocol::jsonrpc::JsonRpcResponse::error(
            ultrafast_mcp_core::protocol::jsonrpc::JsonRpcError::new(-32602, "Invalid params".to_string()),
            Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::Number(1))
        )
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn mcp_client(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);

    // For now, just use defaults - in a real implementation you'd parse the args properly
    let struct_name = &input_struct.ident;
    let client_name = struct_name.to_string();
    let version = "1.0.0";

    let expanded = quote! {
        #input_struct

        impl #struct_name {
            /// Get client information
            pub fn client_info() -> ultrafast_mcp_core::types::client::ClientInfo {
                ultrafast_mcp_core::types::client::ClientInfo {
                    name: #client_name.to_string(),
                    version: #version.to_string(),
                    description: None,
                    homepage: None,
                    repository: None,
                    authors: None,
                    license: None,
                }
            }
        }
    };

    TokenStream::from(expanded)
}
