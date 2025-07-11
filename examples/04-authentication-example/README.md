# UltraFast MCP Authentication Example

This example demonstrates how to use various authentication methods with UltraFast MCP client and server.

## Features Demonstrated

- **Bearer Token Authentication**: JWT-based authentication with scope validation
- **API Key Authentication**: Simple API key-based authentication
- **Basic Authentication**: Username/password authentication
- **Custom Header Authentication**: Flexible custom header-based authentication
- **OAuth 2.1 Authentication**: Full OAuth flow with PKCE
- **Auto-refresh Tokens**: Automatic token refresh for Bearer tokens
- **Server-side Authentication Middleware**: Request validation and scope checking
- **Client-side Authentication Middleware**: Header generation and token management
- **HTTP Transport Authentication**: Authentication with HTTP transport layer

## Authentication Methods Supported

### 1. Bearer Token Authentication

Bearer token authentication uses JWT tokens for secure API access.

```rust
// Server-side
let server = UltraFastServer::new(info, capabilities)
    .with_bearer_auth(
        "your-jwt-secret-key".to_string(),
        vec!["read".to_string(), "write".to_string()],
    );

// Client-side
let client = UltraFastClient::new(info, capabilities)
    .with_bearer_auth("your-access-token".to_string());

// With auto-refresh
let client = UltraFastClient::new(info, capabilities)
    .with_bearer_auth_refresh(
        "your-access-token".to_string(),
        || async {
            // Call your token refresh endpoint
            Ok::<String, AuthError>("new-refreshed-token".to_string())
        },
    );
```

### 2. API Key Authentication

API key authentication uses simple API keys for authentication.

```rust
// Client-side
let client = UltraFastClient::new(info, capabilities)
    .with_api_key_auth("your-api-key".to_string());

// With custom header name
let client = UltraFastClient::new(info, capabilities)
    .with_api_key_auth_custom("your-api-key".to_string(), "X-Custom-API-Key".to_string());
```

### 3. Basic Authentication

Basic authentication uses username/password credentials.

```rust
// Client-side
let client = UltraFastClient::new(info, capabilities)
    .with_basic_auth("username".to_string(), "password".to_string());
```

### 4. Custom Header Authentication

Custom header authentication allows flexible header-based authentication.

```rust
// Client-side
let client = UltraFastClient::new(info, capabilities)
    .with_custom_auth()
    .with_auth(ultrafast_mcp_auth::AuthMethod::custom()
        .with_header("X-Custom-Header".to_string(), "custom-value".to_string())
        .with_header("X-Another-Header".to_string(), "another-value".to_string()));
```

### 5. OAuth 2.1 Authentication

OAuth 2.1 authentication provides full OAuth flow with PKCE.

```rust
// Client-side
let oauth_config = ultrafast_mcp_auth::OAuthConfig {
    client_id: "your-client-id".to_string(),
    client_secret: "your-client-secret".to_string(),
    auth_url: "https://auth.example.com/oauth/authorize".to_string(),
    token_url: "https://auth.example.com/oauth/token".to_string(),
    redirect_uri: "http://localhost:8080/callback".to_string(),
    scopes: vec!["read".to_string(), "write".to_string()],
};

let client = UltraFastClient::new(info, capabilities)
    .with_oauth_auth(oauth_config);
```

## HTTP Transport Authentication

The HTTP transport layer also supports all authentication methods:

```rust
use ultrafast_mcp_transport::streamable_http::client::StreamableHttpClientConfig;

// Bearer token
let config = StreamableHttpClientConfig::default()
    .with_bearer_auth("your-access-token".to_string());

// API key
let config = StreamableHttpClientConfig::default()
    .with_api_key_auth("your-api-key".to_string());

// Basic auth
let config = StreamableHttpClientConfig::default()
    .with_basic_auth("username".to_string(), "password".to_string());

// OAuth
let config = StreamableHttpClientConfig::default()
    .with_oauth_auth(oauth_config);

// Custom headers
let config = StreamableHttpClientConfig::default()
    .with_custom_auth()
    .with_auth_method(ultrafast_mcp_auth::AuthMethod::custom()
        .with_header("X-Custom-Header".to_string(), "custom-value".to_string()));
```

## Server-side Authentication Middleware

The server-side authentication middleware provides request validation and scope checking:

```rust
use ultrafast_mcp_auth::{ServerAuthMiddleware, TokenValidator, AuthContext};

// Create token validator
let token_validator = TokenValidator::new("your-jwt-secret".to_string());

// Create auth middleware
let auth_middleware = ServerAuthMiddleware::new(token_validator)
    .with_required_scopes(vec!["read".to_string(), "write".to_string()]);

// Validate request
let mut headers = HashMap::new();
headers.insert("Authorization".to_string(), "Bearer your-jwt-token".to_string());

match auth_middleware.validate_request(&headers).await {
    Ok(auth_context) => {
        println!("Authentication successful");
        println!("User ID: {:?}", auth_context.user_id);
        println!("Scopes: {:?}", auth_context.scopes);
        println!("Authenticated: {}", auth_context.is_authenticated);
    }
    Err(e) => {
        println!("Authentication failed: {:?}", e);
    }
}
```

## Client-side Authentication Middleware

The client-side authentication middleware handles header generation and token management:

```rust
use ultrafast_mcp_auth::{ClientAuthMiddleware, AuthMethod};

// Create auth middleware with Bearer token
let auth_method = AuthMethod::bearer("your-access-token".to_string());
let mut auth_middleware = ClientAuthMiddleware::new(auth_method);

// Get authentication headers
match auth_middleware.get_headers().await {
    Ok(headers) => {
        for (key, value) in headers {
            println!("{}: {}", key, value);
        }
    }
    Err(e) => {
        println!("Failed to get auth headers: {:?}", e);
    }
}
```

## Running the Example

1. **Enable OAuth feature**:
   ```bash
   cargo run --features oauth
   ```

2. **Run without OAuth feature** (limited functionality):
   ```bash
   cargo run
   ```

## Security Best Practices

### 1. Token Security
- Use short-lived access tokens (15-60 minutes)
- Implement proper token refresh logic
- Store tokens securely (encrypted, not in plain text)
- Validate all tokens before use

### 2. OAuth Security
- Always use PKCE for public clients
- Validate state parameters to prevent CSRF attacks
- Use secure redirect URIs
- Implement proper logout and token revocation

### 3. API Key Security
- Use strong, randomly generated API keys
- Rotate API keys regularly
- Use HTTPS for all API communications
- Implement rate limiting

### 4. Basic Authentication
- Use HTTPS for all communications
- Consider using Basic auth only for internal services
- Implement proper password policies
- Use strong password hashing

### 5. Custom Headers
- Use HTTPS for all communications
- Validate all custom header values
- Implement proper header sanitization
- Use descriptive header names

## Error Handling

The authentication system provides comprehensive error handling:

```rust
use ultrafast_mcp_auth::{AuthError, AuthResult};

async fn handle_auth_error(result: AuthResult<()>) {
    match result {
        Ok(()) => println!("Authentication successful"),
        Err(AuthError::InvalidToken(message)) => {
            eprintln!("Invalid token: {}", message);
            // Handle invalid token
        }
        Err(AuthError::MissingScope { scope }) => {
            eprintln!("Missing scope: {}", scope);
            // Handle permission error
        }
        Err(AuthError::NetworkError(message)) => {
            eprintln!("Network error: {}", message);
            // Handle network error
        }
        Err(e) => {
            eprintln!("Authentication error: {:?}", e);
            // Handle other errors
        }
    }
}
```

## Integration with Existing Code

The authentication system is designed to integrate seamlessly with existing UltraFast MCP code:

```rust
// Existing server code
let server = UltraFastServer::new(info, capabilities)
    .with_tool_handler(Arc::new(my_tool_handler))
    .with_resource_handler(Arc::new(my_resource_handler))
    // Add authentication
    .with_bearer_auth("your-jwt-secret".to_string(), vec!["read".to_string()]);

// Existing client code
let client = UltraFastClient::new(info, capabilities)
    .with_elicitation_handler(Arc::new(my_elicitation_handler))
    // Add authentication
    .with_bearer_auth("your-access-token".to_string());
```

## Performance Considerations

- **Token Caching**: Cache validated tokens to reduce validation overhead
- **Connection Pooling**: Reuse HTTP connections for token requests
- **Async Operations**: All operations are async for non-blocking performance
- **Efficient Validation**: Optimized JWT validation algorithms
- **Minimal Allocations**: Efficient memory usage in hot paths

## Thread Safety

All authentication components are designed to be thread-safe:
- OAuth clients are `Send + Sync`
- Token validators are stateless and thread-safe
- PKCE utilities are stateless and thread-safe
- No mutable global state is used

## License

This example is licensed under the MIT OR Apache-2.0 license. 