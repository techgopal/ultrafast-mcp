# ultrafast-mcp-auth

Authentication and authorization for ULTRAFAST MCP.

This crate provides comprehensive authentication and authorization support for MCP servers and clients, including OAuth 2.1, JWT tokens, and custom authentication schemes.

## Features

- **OAuth 2.1 Support**: Full OAuth 2.1 implementation with PKCE
- **JWT Tokens**: JSON Web Token generation and validation
- **Multiple Providers**: Support for various OAuth providers
- **Secure Storage**: Secure token storage and management
- **Token Refresh**: Automatic token refresh handling
- **Custom Schemes**: Extensible authentication framework
- **Async Support**: Full async/await support with Tokio

## Usage

### OAuth Authentication

```rust
use ultrafast_mcp_auth::oauth::{OAuthProvider, OAuthConfig};

let config = OAuthConfig::new()
    .with_client_id("your-client-id")
    .with_client_secret("your-client-secret")
    .with_redirect_uri("http://localhost:8080/callback")
    .with_scopes(vec!["read", "write"]);

let provider = OAuthProvider::new(config);
let auth_url = provider.get_authorization_url().await?;
```

### JWT Token Validation

```rust
use ultrafast_mcp_auth::jwt::{JwtValidator, JwtConfig};

let config = JwtConfig::new()
    .with_secret("your-secret-key")
    .with_issuer("your-issuer")
    .with_audience("your-audience");

let validator = JwtValidator::new(config);
let claims = validator.validate_token(&token).await?;
```

## Features

- `oauth` - Enables OAuth 2.1 authentication (default)

## Dependencies

- `oauth2` - OAuth 2.1 implementation
- `jsonwebtoken` - JWT token handling
- `reqwest` - HTTP client for token requests
- `tokio` - Async runtime

## License

MIT OR Apache-2.0 