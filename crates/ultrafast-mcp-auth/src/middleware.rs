use crate::{error::AuthError, types::TokenClaims, AuthMethod, TokenValidator};
use base64::Engine;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Authentication context for requests
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Option<String>,
    pub scopes: Vec<String>,
    pub claims: Option<TokenClaims>,
    pub auth_method: AuthMethod,
    pub is_authenticated: bool,
}

impl AuthContext {
    pub fn new() -> Self {
        Self {
            user_id: None,
            scopes: Vec::new(),
            claims: None,
            auth_method: AuthMethod::None,
            is_authenticated: false,
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    pub fn with_claims(mut self, claims: TokenClaims) -> Self {
        self.claims = Some(claims);
        self
    }

    pub fn with_auth_method(mut self, auth_method: AuthMethod) -> Self {
        self.auth_method = auth_method;
        self
    }

    pub fn authenticated(mut self) -> Self {
        self.is_authenticated = true;
        self
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(&scope.to_string())
    }

    pub fn has_any_scope(&self, scopes: &[String]) -> bool {
        scopes.iter().any(|scope| self.has_scope(scope))
    }

    pub fn has_all_scopes(&self, scopes: &[String]) -> bool {
        scopes.iter().all(|scope| self.has_scope(scope))
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Server-side authentication middleware
pub struct ServerAuthMiddleware {
    token_validator: Arc<TokenValidator>,
    required_scopes: Vec<String>,
    auth_enabled: bool,
    session_store: Arc<RwLock<HashMap<String, AuthContext>>>,
}

impl ServerAuthMiddleware {
    pub fn new(token_validator: TokenValidator) -> Self {
        Self {
            token_validator: Arc::new(token_validator),
            required_scopes: Vec::new(),
            auth_enabled: true,
            session_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_required_scopes(mut self, scopes: Vec<String>) -> Self {
        self.required_scopes = scopes;
        self
    }

    pub fn with_auth_enabled(mut self, enabled: bool) -> Self {
        self.auth_enabled = enabled;
        self
    }

    /// Validate authentication headers and return auth context
    pub async fn validate_request(
        &self,
        headers: &HashMap<String, String>,
    ) -> Result<AuthContext, AuthError> {
        if !self.auth_enabled {
            return Ok(AuthContext::new().authenticated());
        }

        // Check for Authorization header
        if let Some(auth_header) = headers.get("Authorization") {
            return self.validate_auth_header(auth_header).await;
        }

        // Check for API key headers
        for (key, value) in headers {
            if key.to_lowercase().contains("api-key") || key.to_lowercase().contains("x-api-key") {
                return self.validate_api_key(key, value).await;
            }
        }

        // No authentication found
        if self.required_scopes.is_empty() {
            // If no scopes required, allow unauthenticated access
            Ok(AuthContext::new())
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    /// Validate Authorization header
    async fn validate_auth_header(&self, auth_header: &str) -> Result<AuthContext, AuthError> {
        if auth_header.starts_with("Bearer ") {
            self.validate_bearer_token(auth_header).await
        } else if auth_header.starts_with("Basic ") {
            self.validate_basic_auth(auth_header).await
        } else {
            Err(AuthError::InvalidToken(
                "Unsupported authorization scheme".to_string(),
            ))
        }
    }

    /// Validate Bearer token
    async fn validate_bearer_token(&self, auth_header: &str) -> Result<AuthContext, AuthError> {
        let token = crate::validation::extract_bearer_token(auth_header)?;

        // Validate JWT token
        let claims = self.token_validator.validate_token(token).await?;

        // Check required scopes
        if !self.required_scopes.is_empty() {
            self.token_validator
                .validate_scopes(&claims, &self.required_scopes)?;
        }

        let scopes = claims
            .scope
            .as_ref()
            .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let auth_context = AuthContext::new()
            .with_user_id(claims.sub.clone())
            .with_scopes(scopes)
            .with_claims(claims)
            .with_auth_method(AuthMethod::bearer(token.to_string()))
            .authenticated();

        debug!(
            "Bearer token validated for user: {}",
            auth_context
                .user_id
                .as_ref()
                .unwrap_or(&"unknown".to_string())
        );
        Ok(auth_context)
    }

    /// Validate Basic authentication
    async fn validate_basic_auth(&self, auth_header: &str) -> Result<AuthContext, AuthError> {
        // Extract and decode basic auth credentials
        let encoded = auth_header
            .strip_prefix("Basic ")
            .ok_or_else(|| AuthError::InvalidToken("Invalid Basic auth format".to_string()))?;

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|_| AuthError::InvalidToken("Invalid Basic auth encoding".to_string()))?;

        let credentials = String::from_utf8(decoded)
            .map_err(|_| AuthError::InvalidToken("Invalid Basic auth credentials".to_string()))?;

        let parts: Vec<&str> = credentials.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AuthError::InvalidToken(
                "Invalid Basic auth format".to_string(),
            ));
        }

        let username = parts[0];
        let password = parts[1];

        // In a real implementation, you would validate against a user database
        // For now, we'll use a simple validation
        if username.is_empty() || password.is_empty() {
            return Err(AuthError::InvalidCredentials);
        }

        let auth_context = AuthContext::new()
            .with_user_id(username.to_string())
            .with_auth_method(AuthMethod::basic(
                username.to_string(),
                password.to_string(),
            ))
            .authenticated();

        debug!("Basic auth validated for user: {}", username);
        Ok(auth_context)
    }

    /// Validate API key
    async fn validate_api_key(
        &self,
        _header_name: &str,
        api_key: &str,
    ) -> Result<AuthContext, AuthError> {
        if api_key.is_empty() {
            return Err(AuthError::InvalidCredentials);
        }

        // In a real implementation, you would validate the API key against a database
        // For now, we'll accept any non-empty API key
        let auth_context = AuthContext::new()
            .with_user_id(format!("api_user_{}", &api_key[..8.min(api_key.len())]))
            .with_auth_method(AuthMethod::api_key(api_key.to_string()))
            .authenticated();

        debug!(
            "API key validated for user: {}",
            auth_context
                .user_id
                .as_ref()
                .unwrap_or(&"unknown".to_string())
        );
        Ok(auth_context)
    }

    /// Store session authentication context
    pub async fn store_session(&self, session_id: String, auth_context: AuthContext) {
        let mut sessions = self.session_store.write().await;
        sessions.insert(session_id, auth_context);
    }

    /// Get session authentication context
    pub async fn get_session(&self, session_id: &str) -> Option<AuthContext> {
        let sessions = self.session_store.read().await;
        sessions.get(session_id).cloned()
    }

    /// Remove session authentication context
    pub async fn remove_session(&self, session_id: &str) {
        let mut sessions = self.session_store.write().await;
        sessions.remove(session_id);
    }

    /// Check if user has required scopes
    pub fn check_scopes(
        &self,
        auth_context: &AuthContext,
        required_scopes: &[String],
    ) -> Result<(), AuthError> {
        if required_scopes.is_empty() {
            return Ok(());
        }

        if !auth_context.has_all_scopes(required_scopes) {
            let missing_scopes: Vec<String> = required_scopes
                .iter()
                .filter(|scope| !auth_context.has_scope(scope))
                .cloned()
                .collect();

            return Err(AuthError::MissingScope {
                scope: missing_scopes.join(", "),
            });
        }

        Ok(())
    }
}

/// Client-side authentication middleware
pub struct ClientAuthMiddleware {
    auth_method: AuthMethod,
    auto_refresh: bool,
}

impl ClientAuthMiddleware {
    pub fn new(auth_method: AuthMethod) -> Self {
        Self {
            auth_method,
            auto_refresh: true,
        }
    }

    pub fn with_auto_refresh(mut self, enabled: bool) -> Self {
        self.auto_refresh = enabled;
        self
    }

    /// Get authentication headers for outgoing requests
    pub async fn get_headers(&mut self) -> Result<HashMap<String, String>, AuthError> {
        // Refresh token if needed and auto-refresh is enabled
        if self.auto_refresh && self.auth_method.requires_refresh() {
            if let Err(e) = self.auth_method.refresh().await {
                warn!("Failed to refresh authentication: {:?}", e);
            }
        }

        self.auth_method.get_headers().await
    }

    /// Update authentication method
    pub fn with_auth_method(mut self, auth_method: AuthMethod) -> Self {
        self.auth_method = auth_method;
        self
    }

    /// Get current authentication method
    pub fn get_auth_method(&self) -> &AuthMethod {
        &self.auth_method
    }
}

// Remove the conflicting From implementation - it's not needed since AuthError is already the same type

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bearer_auth_validation() {
        let validator = TokenValidator::new("test_secret".to_string());
        let middleware = ServerAuthMiddleware::new(validator)
            .with_required_scopes(vec!["read".to_string(), "write".to_string()]);

        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Bearer invalid_token".to_string(),
        );

        let result = middleware.validate_request(&headers).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_basic_auth_validation() {
        let validator = TokenValidator::new("test_secret".to_string());
        let middleware = ServerAuthMiddleware::new(validator);

        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Basic dXNlcjpwYXNz".to_string(),
        ); // user:pass

        let result = middleware.validate_request(&headers).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_api_key_validation() {
        let validator = TokenValidator::new("test_secret".to_string());
        let middleware = ServerAuthMiddleware::new(validator);

        let mut headers = HashMap::new();
        headers.insert("X-API-Key".to_string(), "test_api_key".to_string());

        let result = middleware.validate_request(&headers).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_auth_context_scopes() {
        let context = AuthContext::new().with_scopes(vec!["read".to_string(), "write".to_string()]);

        assert!(context.has_scope("read"));
        assert!(context.has_scope("write"));
        assert!(!context.has_scope("delete"));

        assert!(context.has_any_scope(&["read".to_string(), "delete".to_string()]));
        assert!(context.has_all_scopes(&["read".to_string(), "write".to_string()]));
        assert!(!context.has_all_scopes(&["read".to_string(), "delete".to_string()]));
    }
}
