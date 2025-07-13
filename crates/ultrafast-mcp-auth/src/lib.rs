//! # UltraFast MCP Authentication
//!
//! Comprehensive authentication and authorization support for the Model Context Protocol (MCP).
//!
//! This crate provides robust authentication mechanisms for MCP servers and clients,
//! including OAuth 2.1 support, PKCE (Proof Key for Code Exchange), and token validation.
//! It's designed to be secure, flexible, and easy to integrate into MCP applications.
//!
//! ## Overview
//!
//! The UltraFast MCP Authentication crate provides:
//!
//! - **OAuth 2.1 Support**: Complete OAuth 2.1 implementation with PKCE
//! - **Token Management**: Secure token storage, validation, and rotation
//! - **PKCE Support**: Proof Key for Code Exchange for enhanced security
//! - **Session Management**: Secure session handling and management
//! - **Validation**: Comprehensive token and credential validation
//! - **Security**: Industry-standard security practices and protocols
//!
//! ## Key Features
//!
//! ### OAuth 2.1 Implementation
//! - **Authorization Code Flow**: Complete OAuth 2.1 authorization code flow
//! - **PKCE Support**: Proof Key for Code Exchange for public clients
//! - **Token Refresh**: Automatic token refresh and rotation
//! - **Scope Management**: Fine-grained permission and scope handling
//! - **State Validation**: CSRF protection through state parameter validation
//!
//! ### Security Features
//! - **PKCE**: Enhanced security for public clients
//! - **State Validation**: Protection against CSRF attacks
//! - **Token Validation**: Comprehensive token verification
//! - **Secure Storage**: Encrypted token storage
//! - **Session Management**: Secure session handling
//!
//! ### Token Management
//! - **Access Tokens**: Short-lived access token handling
//! - **Refresh Tokens**: Long-lived refresh token management
//! - **ID Tokens**: OpenID Connect ID token support
//! - **Token Validation**: JWT validation and verification
//! - **Token Rotation**: Automatic token refresh and rotation
//!
//! ## Modules
//!
//! - **[`oauth`]**: OAuth 2.1 client implementation and flow management
//! - **[`pkce`]**: PKCE (Proof Key for Code Exchange) utilities
//! - **[`types`]**: Authentication-related type definitions
//! - **[`validation`]**: Token and credential validation utilities
//! - **[`error`]**: Authentication-specific error types
//!
//! ## Usage Examples
//!
//! ### Basic OAuth 2.1 Flow
//!
//! ```rust,ignore
//! use ultrafast_mcp_auth::{
//!     OAuthClient, OAuthConfig, generate_pkce_params,
//!     AuthResult, AuthError
//! };
//! use ultrafast_mcp_core::utils::identifiers::{generate_session_id, generate_state};
//!
//! #[tokio::main]
//! async fn main() -> AuthResult<()> {
//!     // Create OAuth configuration
//!     let config = OAuthConfig {
//!         client_id: "your-client-id".to_string(),
//!         client_secret: "your-client-secret".to_string(),
//!         auth_url: "https://auth.example.com/oauth/authorize".to_string(),
//!         token_url: "https://auth.example.com/oauth/token".to_string(),
//!         redirect_uri: "http://localhost:8080/callback".to_string(),
//!         scopes: vec!["read".to_string(), "write".to_string()],
//!     };
//!
//!     // Create OAuth client
//!     let client = OAuthClient::from_config(config.clone());
//!
//!     // Generate PKCE parameters
//!     let pkce_params = generate_pkce_params()?;
//!
//!     // Generate state parameter for CSRF protection
//!     let state = generate_state();
//!
//!     // Create authorization URL
//!     let auth_url = client.get_authorization_url_with_pkce(state, pkce_params.clone()).await?;
//!     println!("Authorization URL: {}", auth_url);
//!
//!     // After user authorization, exchange code for tokens
//!     let code = "authorization_code_from_callback";
//!     let tokens = client.exchange_code_for_token(
//!         &config.token_url,
//!         &config.client_id,
//!         Some(&config.client_secret),
//!         &config.redirect_uri,
//!         code,
//!         &pkce_params.code_verifier
//!     ).await?;
//!
//!     println!("Access token: {}", tokens.access_token);
//!     if let Some(refresh_token) = &tokens.refresh_token {
//!         println!("Refresh token: {}", refresh_token);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Token Validation
//!
//! ```rust
//! use ultrafast_mcp_auth::{TokenValidator, AuthResult, AuthError};
//!
//! async fn validate_request(token: &str) -> AuthResult<()> {
//!     // Create token validator with secret
//!     let validator = TokenValidator::new("your-jwt-secret".to_string());
//!
//!     // Validate the token
//!     let claims = validator.validate_token(token).await?;
//!
//!     // Check if token has required scopes
//!     if let Some(scope) = &claims.scope {
//!         if !scope.contains("read") {
//!             return Err(AuthError::MissingScope {
//!                 scope: "read".to_string()
//!             });
//!         }
//!     }
//!
//!     println!("Token is valid for user: {}", claims.sub);
//!     Ok(())
//! }
//! ```
//!
//! ### PKCE Implementation
//!
//! ```rust,ignore
//! use ultrafast_mcp_auth::{generate_pkce_params, AuthResult};
//! use ultrafast_mcp_core::utils::identifiers::{generate_session_id, generate_state};
//!
//! fn setup_pkce_flow() -> AuthResult<(String, String, String)> {
//!     // Generate PKCE parameters
//!     let pkce_params = generate_pkce_params()?;
//!
//!     // Generate session ID for tracking
//!     let session_id = generate_session_id();
//!
//!     // Generate state parameter
//!     let state = generate_state();
//!
//!     println!("Code verifier: {}", pkce_params.code_verifier);
//!     println!("Code challenge: {}", pkce_params.code_challenge);
//!     println!("Session ID: {}", session_id);
//!     println!("State: {}", state);
//!
//!     Ok((pkce_params.code_verifier, session_id, state))
//! }
//! ```
//!
//! ### OAuth Client with Refresh
//!
//! ```rust
//! use ultrafast_mcp_auth::{OAuthClient, OAuthConfig, TokenResponse, AuthResult};
//!
//! async fn handle_token_refresh(client: &OAuthClient, refresh_token: &str, token_url: &str) -> AuthResult<()> {
//!     // Refresh the access token
//!     let new_tokens = client.refresh_token(token_url, &client.client_id(), Some(&client.client_secret()), refresh_token).await?;
//!
//!     // Store the new tokens securely
//!     store_tokens_securely(&new_tokens).await?;
//!
//!     println!("Tokens refreshed successfully");
//!     Ok(())
//! }
//!
//! async fn store_tokens_securely(tokens: &TokenResponse) -> AuthResult<()> {
//!     // Implement secure token storage
//!     // This could involve encryption, secure key storage, etc.
//!     Ok(())
//! }
//! ```
//!
//! ## OAuth 2.1 Flow
//!
//! The crate implements the complete OAuth 2.1 authorization code flow:
//!
//! ```text
//! Client                    Authorization Server
//!   |                              |
//!   |-- Authorization Request ---->|
//!   |<-- Authorization Code -------|
//!   |                              |
//!   |-- Token Request ------------>|
//!   |<-- Access Token + Refresh ---|
//!   |                              |
//!   |-- API Request -------------->|
//!   |<-- Protected Resource -------|
//! ```
//!
//! ### PKCE Flow
//!
//! For enhanced security, the crate supports PKCE:
//!
//! ```text
//! 1. Generate code_verifier (random string)
//! 2. Generate code_challenge (SHA256 hash of code_verifier)
//! 3. Send code_challenge in authorization request
//! 4. Send code_verifier in token request
//! 5. Server validates code_challenge against code_verifier
//! ```
//!
//! ## Security Features
//!
//! ### CSRF Protection
//! - **State Parameter**: Random state parameter for each authorization request
//! - **Validation**: Server validates state parameter in callback
//! - **Session Binding**: State parameter bound to user session
//!
//! ### Token Security
//! - **Short-lived Access Tokens**: Access tokens expire quickly
//! - **Secure Refresh Tokens**: Long-lived refresh tokens with rotation
//! - **Token Validation**: Comprehensive JWT validation
//! - **Scope Validation**: Fine-grained permission checking
//!
//! ### PKCE Security
//! - **Code Verifier**: Random string generated by client
//! - **Code Challenge**: SHA256 hash of code verifier
//! - **Validation**: Server validates challenge against verifier
//! - **Public Client Security**: Enhanced security for public clients
//!
//! ## Configuration
//!
//! ### OAuth Configuration
//! ```rust
//! use ultrafast_mcp_auth::OAuthConfig;
//!
//! let config = OAuthConfig {
//!     client_id: "your-client-id".to_string(),
//!     client_secret: "your-client-secret".to_string(),
//!     auth_url: "https://auth.example.com/oauth/authorize".to_string(),
//!     token_url: "https://auth.example.com/oauth/token".to_string(),
//!     redirect_uri: "http://localhost:8080/callback".to_string(),
//!     scopes: vec!["read".to_string(), "write".to_string()],
//! };
//! ```
//!
//! ### Token Validator Configuration
//! ```rust
//! use ultrafast_mcp_auth::TokenValidator;
//!
//! let validator = TokenValidator::new("your-jwt-secret".to_string());
//! ```
//!
//! ## Error Handling
//!
//! The crate provides comprehensive error handling:
//!
//! ```rust
//! use ultrafast_mcp_auth::{AuthError, AuthResult};
//!
//! async fn handle_auth_error(result: AuthResult<()>) {
//!     match result {
//!         Ok(()) => println!("Authentication successful"),
//!         Err(AuthError::InvalidToken(message)) => {
//!             eprintln!("Invalid token: {}", message);
//!             // Handle invalid token
//!         }
//!         Err(AuthError::MissingScope { scope }) => {
//!             eprintln!("Missing scope: {}", scope);
//!             // Handle permission error
//!         }
//!         Err(AuthError::NetworkError(message)) => {
//!             eprintln!("Network error: {}", message);
//!             // Handle network error
//!         }
//!         Err(e) => {
//!             eprintln!("Authentication error: {:?}", e);
//!             // Handle other errors
//!         }
//!     }
//! }
//! ```
//!
//! ## Best Practices
//!
//! ### Security
//! - Always use PKCE for public clients
//! - Validate state parameters to prevent CSRF attacks
//! - Store tokens securely (encrypted, not in plain text)
//! - Use short-lived access tokens
//! - Implement proper token refresh logic
//! - Validate all tokens before use
//!
//! ### Implementation
//! - Handle all error cases gracefully
//! - Implement proper token refresh logic
//! - Use secure random number generation
//! - Validate all inputs and outputs
//! - Log authentication events for audit
//!
//! ### Integration
//! - Integrate with your application's session management
//! - Implement proper error handling and user feedback
//! - Use appropriate timeouts for network requests
//! - Handle network failures gracefully
//! - Implement proper logout and token revocation
//!
//! ## Performance Considerations
//!
//! - **Token Caching**: Cache validated tokens to reduce validation overhead
//! - **Connection Pooling**: Reuse HTTP connections for token requests
//! - **Async Operations**: All operations are async for non-blocking performance
//! - **Efficient Validation**: Optimized JWT validation algorithms
//! - **Minimal Allocations**: Efficient memory usage in hot paths
//!
//! ## Thread Safety
//!
//! All authentication components are designed to be thread-safe:
//! - OAuth clients are `Send + Sync`
//! - Token validators are stateless and thread-safe
//! - PKCE utilities are stateless and thread-safe
//! - No mutable global state is used
//!
//! ## Examples
//!
//! See the `examples/` directory for complete working examples:
//! - Basic OAuth 2.1 flow
//! - PKCE implementation
//! - Token validation
//! - Refresh token handling
//! - Integration with MCP servers and clients

pub mod error;
pub mod middleware;
pub mod oauth;
pub mod pkce;
pub mod types;
pub mod validation;

pub use error::AuthError;
pub use oauth::OAuthClient;
pub use pkce::generate_pkce_params;
// generate_session_id and generate_state are now available directly from ultrafast_mcp_core::utils
pub use middleware::{AuthContext, ClientAuthMiddleware, ServerAuthMiddleware};
pub use types::*;
pub use validation::{TokenValidator, extract_bearer_token};

/// Result type for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;
