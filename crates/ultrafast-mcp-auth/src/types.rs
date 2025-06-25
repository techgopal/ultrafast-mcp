use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
