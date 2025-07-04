use thiserror::Error;

/// Authentication and authorization errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("OAuth authentication failed: {message}")]
    OAuthError { message: String },

    #[error("Token validation failed: {reason}")]
    TokenValidationError { reason: String },

    #[error("Invalid client credentials")]
    InvalidCredentials,

    #[error("Expired token")]
    ExpiredToken,

    #[error("Invalid token audience: expected {expected}, got {actual}")]
    InvalidAudience { expected: String, actual: String },

    #[error("Missing required scope: {scope}")]
    MissingScope { scope: String },

    #[error("PKCE challenge failed")]
    PkceChallengeFailed,

    #[error("Authorization server error: {error}")]
    AuthorizationServerError { error: String },

    #[error("Token exchange error: {error}")]
    TokenExchangeError { error: String },

    #[error("Invalid token")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid client: {0}")]
    InvalidClient(String),

    #[error("Invalid grant: {0}")]
    InvalidGrant(String),

    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    #[error("Unauthorized client")]
    UnauthorizedClient,

    #[error("Unsupported grant type: {0}")]
    UnsupportedGrantType(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Network error during authentication: {source}")]
    ReqwestError {
        #[from]
        source: reqwest::Error,
    },

    #[error("JWT error: {source}")]
    JwtError {
        #[from]
        source: jsonwebtoken::errors::Error,
    },

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("Invalid URL: {source}")]
    UrlError {
        #[from]
        source: url::ParseError,
    },
}

impl PartialEq for AuthError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AuthError::OAuthError { message: a }, AuthError::OAuthError { message: b }) => a == b,
            (
                AuthError::TokenValidationError { reason: a },
                AuthError::TokenValidationError { reason: b },
            ) => a == b,
            (AuthError::InvalidCredentials, AuthError::InvalidCredentials) => true,
            (AuthError::ExpiredToken, AuthError::ExpiredToken) => true,
            (
                AuthError::InvalidAudience {
                    expected: e1,
                    actual: a1,
                },
                AuthError::InvalidAudience {
                    expected: e2,
                    actual: a2,
                },
            ) => e1 == e2 && a1 == a2,
            (AuthError::MissingScope { scope: a }, AuthError::MissingScope { scope: b }) => a == b,
            (AuthError::PkceChallengeFailed, AuthError::PkceChallengeFailed) => true,
            (
                AuthError::AuthorizationServerError { error: a },
                AuthError::AuthorizationServerError { error: b },
            ) => a == b,
            (
                AuthError::TokenExchangeError { error: a },
                AuthError::TokenExchangeError { error: b },
            ) => a == b,
            (AuthError::InvalidToken(a), AuthError::InvalidToken(b)) => a == b,
            (AuthError::TokenExpired, AuthError::TokenExpired) => true,
            (AuthError::InvalidClient(a), AuthError::InvalidClient(b)) => a == b,
            (AuthError::InvalidGrant(a), AuthError::InvalidGrant(b)) => a == b,
            (AuthError::InvalidScope(a), AuthError::InvalidScope(b)) => a == b,
            (AuthError::UnauthorizedClient, AuthError::UnauthorizedClient) => true,
            (AuthError::UnsupportedGrantType(a), AuthError::UnsupportedGrantType(b)) => a == b,
            (AuthError::InvalidRequest(a), AuthError::InvalidRequest(b)) => a == b,
            (AuthError::ServerError(a), AuthError::ServerError(b)) => a == b,
            (AuthError::NetworkError(a), AuthError::NetworkError(b)) => a == b,
            // For errors with external types, compare by their string representation
            (AuthError::ReqwestError { source: a }, AuthError::ReqwestError { source: b }) => {
                a.to_string() == b.to_string()
            }
            (AuthError::JwtError { source: a }, AuthError::JwtError { source: b }) => {
                a.to_string() == b.to_string()
            }
            (
                AuthError::SerializationError { source: a },
                AuthError::SerializationError { source: b },
            ) => a.to_string() == b.to_string(),
            (AuthError::UrlError { source: a }, AuthError::UrlError { source: b }) => {
                a.to_string() == b.to_string()
            }
            _ => false,
        }
    }
}
