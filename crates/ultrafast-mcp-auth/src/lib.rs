pub mod error;
pub mod types;
pub mod oauth;
pub mod pkce;
pub mod validation;

pub use error::AuthError;
pub use types::*;
pub use oauth::OAuthClient;
pub use pkce::{generate_pkce_params, generate_state, generate_session_id};
pub use validation::{TokenValidator, extract_bearer_token};

/// Result type for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;
