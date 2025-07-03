pub mod error;
pub mod oauth;
pub mod pkce;
pub mod types;
pub mod validation;

pub use error::AuthError;
pub use oauth::OAuthClient;
pub use pkce::{generate_pkce_params, generate_session_id, generate_state};
pub use types::*;
pub use validation::{extract_bearer_token, TokenValidator};

/// Result type for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;
