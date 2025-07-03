//! Procedural macros for the ULTRAFAST MCP implementation
//!
//! This crate provides convenience macros for:
//! - Automatic schema generation
//! - Tool registration
//! - Server setup
//! - Client configuration

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, ItemFn, ItemStruct};

/// Derive macro for automatic JSON Schema generation
///
/// # Example
/// ```compile_fail
/// use ultrafast_mcp_macros::McpSchema;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(McpSchema, Serialize, Deserialize)]
/// struct MyTool {
///     name: String,
///     value: i32,
/// }
/// ```
#[proc_macro_derive(McpSchema, attributes(mcp))]
pub fn derive_mcp_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let schema_impl = match &input.data {
        Data::Struct(data_struct) => {
            let field_schemas =
                match &data_struct.fields {
                    Fields::Named(fields) => {
                        let field_entries = fields.named.iter().filter_map(|field| {
                        let field_name = field.ident.as_ref()?; // Skip fields without a name
                        let field_name_str = field_name.to_string();
                        Some(quote! {
                            properties.insert(#field_name_str.to_string(), serde_json::json!({
                                "type": "string" // TODO: Infer actual type
                            }));
                        })
                    }).collect::<Vec<_>>();
                        quote! {
                            let mut properties = std::collections::HashMap::new();
                            #(#field_entries)*
                            properties
                        }
                    }
                    Fields::Unnamed(_) => {
                        quote! {
                            std::collections::HashMap::new() // TODO: Handle tuple structs
                        }
                    }
                    Fields::Unit => {
                        quote! {
                            std::collections::HashMap::new()
                        }
                    }
                };

            quote! {
                impl #impl_generics ultrafast_mcp_core::schema::McpSchema for #name #ty_generics #where_clause {
                    fn schema() -> serde_json::Value {
                        let properties = #field_schemas;
                        serde_json::json!({
                            "type": "object",
                            "properties": properties,
                            "additionalProperties": false
                        })
                    }

                    fn schema_name() -> String {
                        stringify!(#name).to_string()
                    }
                }
            }
        }
        Data::Enum(_) => {
            quote! {
                impl #impl_generics ultrafast_mcp_core::schema::McpSchema for #name #ty_generics #where_clause {
                    fn schema() -> serde_json::Value {
                        serde_json::json!({
                            "type": "string",
                            "enum": [] // TODO: Extract enum variants
                        })
                    }

                    fn schema_name() -> String {
                        stringify!(#name).to_string()
                    }
                }
            }
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(&input, "McpSchema cannot be derived for unions")
                .to_compile_error()
                .into();
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

    let expanded = quote! {
        #input_fn

        // Generate tool registration function
        pub fn register_tool() -> ultrafast_mcp_core::types::tools::Tool {
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
