use crate::error::AuthError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type alias for refresh function to reduce complexity
pub type RefreshFn = Box<
    dyn Fn()
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, AuthError>> + Send>>
        + Send
        + Sync,
>;

/// OAuth 2.1 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

/// OAuth token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// PKCE parameters for OAuth 2.1
#[derive(Debug, Clone)]
pub struct PkceParams {
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

/// Bearer token authentication
pub struct BearerAuth {
    pub token: String,
    pub refresh_fn: Option<RefreshFn>,
}

impl std::fmt::Debug for BearerAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BearerAuth")
            .field("token", &"***REDACTED***")
            .field(
                "refresh_fn",
                &if self.refresh_fn.is_some() {
                    "Some(Fn)"
                } else {
                    "None"
                },
            )
            .finish()
    }
}

impl Clone for BearerAuth {
    fn clone(&self) -> Self {
        Self {
            token: self.token.clone(),
            refresh_fn: None, // Can't clone function pointers, so we set to None
        }
    }
}

impl BearerAuth {
    pub fn new(token: String) -> Self {
        Self {
            token,
            refresh_fn: None,
        }
    }

    pub fn with_auto_refresh<F, Fut>(mut self, refresh_fn: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<String, AuthError>> + Send + 'static,
    {
        self.refresh_fn = Some(Box::new(move || {
            let fut = refresh_fn();
            Box::pin(fut)
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<String, AuthError>> + Send>,
                >
        }));
        self
    }

    pub fn get_token(&self) -> &str {
        &self.token
    }

    pub async fn refresh_token(&self) -> Result<String, AuthError> {
        if let Some(refresh_fn) = &self.refresh_fn {
            refresh_fn().await
        } else {
            Err(AuthError::InvalidToken(
                "No refresh function configured".to_string(),
            ))
        }
    }
}

/// API Key authentication
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub api_key: String,
    pub header_name: String,
}

impl ApiKeyAuth {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            header_name: "X-API-Key".to_string(),
        }
    }

    pub fn with_header_name(mut self, header_name: String) -> Self {
        self.header_name = header_name;
        self
    }

    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }

    pub fn get_header_name(&self) -> &str {
        &self.header_name
    }
}

/// Basic authentication
#[derive(Debug, Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

impl BasicAuth {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub fn from_credentials(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub fn encode_credentials(&self) -> String {
        use base64::Engine;
        let credentials = format!("{}:{}", self.username, self.password);
        base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes())
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_password(&self) -> &str {
        &self.password
    }
}

/// Custom header authentication
#[derive(Debug, Clone)]
pub struct CustomHeaderAuth {
    pub headers: HashMap<String, String>,
}

impl CustomHeaderAuth {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn with_api_key(self, api_key: String) -> Self {
        self.with_header("X-API-Key".to_string(), api_key)
    }

    pub fn with_bearer_token(self, token: String) -> Self {
        self.with_header("Authorization".to_string(), format!("Bearer {}", token))
    }

    pub fn with_basic_auth(self, username: &str, password: &str) -> Self {
        let basic_auth = BasicAuth::from_credentials(username, password);
        self.with_header(
            "Authorization".to_string(),
            format!("Basic {}", basic_auth.encode_credentials()),
        )
    }

    pub fn get_headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
}

impl Default for CustomHeaderAuth {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified authentication method
pub enum AuthMethod {
    Bearer(BearerAuth),
    OAuth(OAuthConfig),
    ApiKey(ApiKeyAuth),
    Basic(BasicAuth),
    Custom(CustomHeaderAuth),
    None,
}

impl std::fmt::Debug for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::Bearer(bearer) => f.debug_tuple("Bearer").field(bearer).finish(),
            AuthMethod::OAuth(config) => f.debug_tuple("OAuth").field(config).finish(),
            AuthMethod::ApiKey(api_key) => f.debug_tuple("ApiKey").field(api_key).finish(),
            AuthMethod::Basic(basic) => f.debug_tuple("Basic").field(basic).finish(),
            AuthMethod::Custom(custom) => f.debug_tuple("Custom").field(custom).finish(),
            AuthMethod::None => write!(f, "None"),
        }
    }
}

impl Clone for AuthMethod {
    fn clone(&self) -> Self {
        match self {
            AuthMethod::Bearer(bearer) => AuthMethod::Bearer(bearer.clone()),
            AuthMethod::OAuth(config) => AuthMethod::OAuth(config.clone()),
            AuthMethod::ApiKey(api_key) => AuthMethod::ApiKey(api_key.clone()),
            AuthMethod::Basic(basic) => AuthMethod::Basic(basic.clone()),
            AuthMethod::Custom(custom) => AuthMethod::Custom(custom.clone()),
            AuthMethod::None => AuthMethod::None,
        }
    }
}

impl AuthMethod {
    pub fn bearer(token: String) -> Self {
        Self::Bearer(BearerAuth::new(token))
    }

    pub fn oauth(config: OAuthConfig) -> Self {
        Self::OAuth(config)
    }

    pub fn api_key(api_key: String) -> Self {
        Self::ApiKey(ApiKeyAuth::new(api_key))
    }

    pub fn basic(username: String, password: String) -> Self {
        Self::Basic(BasicAuth::new(username, password))
    }

    pub fn custom() -> Self {
        Self::Custom(CustomHeaderAuth::new())
    }

    pub fn none() -> Self {
        Self::None
    }

    /// Get authentication headers for HTTP requests
    pub async fn get_headers(&self) -> Result<HashMap<String, String>, AuthError> {
        let mut headers = HashMap::new();

        match self {
            AuthMethod::Bearer(bearer) => {
                headers.insert(
                    "Authorization".to_string(),
                    format!("Bearer {}", bearer.get_token()),
                );
            }
            AuthMethod::OAuth(_) => {
                // OAuth tokens are handled separately in the OAuth flow
                // This would typically be called after token acquisition
            }
            AuthMethod::ApiKey(api_key) => {
                headers.insert(
                    api_key.get_header_name().to_string(),
                    api_key.get_api_key().to_string(),
                );
            }
            AuthMethod::Basic(basic) => {
                headers.insert(
                    "Authorization".to_string(),
                    format!("Basic {}", basic.encode_credentials()),
                );
            }
            AuthMethod::Custom(custom) => {
                headers.extend(custom.get_headers().clone());
            }
            AuthMethod::None => {
                // No authentication headers
            }
        }

        Ok(headers)
    }

    /// Check if this auth method requires token refresh
    pub fn requires_refresh(&self) -> bool {
        matches!(self, AuthMethod::Bearer(_) | AuthMethod::OAuth(_))
    }

    /// Refresh authentication if needed
    pub async fn refresh(&mut self) -> Result<(), AuthError> {
        match self {
            AuthMethod::Bearer(bearer) => {
                if bearer.refresh_fn.is_some() {
                    let new_token = bearer.refresh_token().await?;
                    bearer.token = new_token;
                }
            }
            AuthMethod::OAuth(_) => {
                // OAuth refresh is handled by OAuthClient
            }
            _ => {
                // Other auth methods don't support refresh
            }
        }
        Ok(())
    }
}

impl Default for AuthMethod {
    fn default() -> Self {
        Self::None
    }
}

/// Authorization server metadata (RFC 8414)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationServerMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: Option<String>,
    pub scopes_supported: Option<Vec<String>>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Option<Vec<String>>,
    pub token_endpoint_auth_methods_supported: Option<Vec<String>>,
    pub code_challenge_methods_supported: Option<Vec<String>>,

    #[serde(flatten)]
    pub additional_metadata: HashMap<String, serde_json::Value>,
}

/// Client registration request (RFC 7591)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRegistrationRequest {
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
    pub client_uri: Option<String>,
    pub logo_uri: Option<String>,
    pub scope: Option<String>,
    pub contacts: Option<Vec<String>>,
    pub tos_uri: Option<String>,
    pub policy_uri: Option<String>,
    pub token_endpoint_auth_method: Option<String>,
    pub grant_types: Option<Vec<String>>,
    pub response_types: Option<Vec<String>>,
}

/// Client registration response (RFC 7591)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRegistrationResponse {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub client_id_issued_at: Option<u64>,
    pub client_secret_expires_at: Option<u64>,
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
    pub client_uri: Option<String>,
    pub logo_uri: Option<String>,
    pub scope: Option<String>,
    pub contacts: Option<Vec<String>>,
    pub tos_uri: Option<String>,
    pub policy_uri: Option<String>,
    pub token_endpoint_auth_method: Option<String>,
    pub grant_types: Option<Vec<String>>,
    pub response_types: Option<Vec<String>>,
}

/// JWT token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,           // Subject
    pub aud: Vec<String>,      // Audience
    pub iss: String,           // Issuer
    pub exp: u64,              // Expiration time
    pub iat: u64,              // Issued at
    pub nbf: Option<u64>,      // Not before
    pub jti: Option<String>,   // JWT ID
    pub scope: Option<String>, // OAuth scopes

    #[serde(flatten)]
    pub additional_claims: HashMap<String, serde_json::Value>,
}
