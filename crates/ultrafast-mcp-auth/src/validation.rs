use crate::{error::AuthError, types::TokenClaims};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

/// Token validator for JWT access tokens
#[derive(Clone)]
pub struct TokenValidator {
    validation: Validation,
    decoding_key: Option<DecodingKey>,
    secret: String,
}

impl TokenValidator {
    /// Create a new token validator with a secret
    pub fn new(secret: String) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Self {
            validation,
            decoding_key: Some(DecodingKey::from_secret(secret.as_ref())),
            secret,
        }
    }

    /// Get the secret (for testing)
    pub fn get_secret(&self) -> &str {
        &self.secret
    }

    /// Set the decoding key for JWT verification
    pub fn with_decoding_key(mut self, key: DecodingKey) -> Self {
        self.decoding_key = Some(key);
        self
    }

    /// Set required audience
    pub fn with_audience<T: ToString>(mut self, audience: T) -> Self {
        self.validation.set_audience(&[audience.to_string()]);
        self
    }

    /// Set required issuer
    pub fn with_issuer<T: ToString>(mut self, issuer: T) -> Self {
        self.validation.set_issuer(&[issuer.to_string()]);
        self
    }

    /// Validate a JWT access token
    pub async fn validate_token(&self, token: &str) -> Result<TokenClaims, AuthError> {
        let decoding_key =
            self.decoding_key
                .as_ref()
                .ok_or_else(|| AuthError::TokenValidationError {
                    reason: "No decoding key configured".to_string(),
                })?;

        let token_data = decode::<TokenClaims>(token, decoding_key, &self.validation)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(token_data.claims)
    }

    /// Validate token audience specifically (RFC 8707)
    pub fn validate_audience(
        &self,
        claims: &TokenClaims,
        expected_audience: &str,
    ) -> Result<(), AuthError> {
        if !claims.aud.contains(&expected_audience.to_string()) {
            return Err(AuthError::InvalidAudience {
                expected: expected_audience.to_string(),
                actual: claims.aud.join(", "),
            });
        }
        Ok(())
    }

    /// Validate required scopes
    pub fn validate_scopes(
        &self,
        claims: &TokenClaims,
        required_scopes: &[String],
    ) -> Result<(), AuthError> {
        let token_scopes = claims
            .scope
            .as_ref()
            .map(|s| s.split_whitespace().collect::<Vec<_>>())
            .unwrap_or_default();

        for required_scope in required_scopes {
            if !token_scopes.contains(&required_scope.as_str()) {
                return Err(AuthError::MissingScope {
                    scope: required_scope.clone(),
                });
            }
        }

        Ok(())
    }
}

impl Default for TokenValidator {
    fn default() -> Self {
        Self::new("".to_string())
    }
}

/// Extract bearer token from Authorization header
pub fn extract_bearer_token(auth_header: &str) -> Result<&str, AuthError> {
    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidToken("Not a Bearer token".to_string()));
    }
    let token = auth_header.strip_prefix("Bearer ").unwrap();
    let token = token.trim();
    if token.is_empty() {
        return Err(AuthError::InvalidToken("Empty token".to_string()));
    }

    // Validate token length (prevent extremely long tokens)
    const MAX_TOKEN_LENGTH: usize = 4096;
    if token.len() > MAX_TOKEN_LENGTH {
        return Err(AuthError::InvalidToken(format!(
            "Token too long: {} characters (max: {})",
            token.len(),
            MAX_TOKEN_LENGTH
        )));
    }

    Ok(token)
}

/// Extract and validate JWT bearer token from Authorization header
pub fn extract_jwt_bearer_token(auth_header: &str) -> Result<&str, AuthError> {
    let token = extract_bearer_token(auth_header)?;

    // Basic JWT format validation (should contain exactly 3 parts separated by dots)
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthError::InvalidToken(
            "Invalid JWT format: expected 3 parts".to_string(),
        ));
    }

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token() {
        // Test simple tokens (for backward compatibility)
        assert_eq!(extract_bearer_token("Bearer abc123"), Ok("abc123"));
        assert_eq!(extract_bearer_token("Bearer  abc123  "), Ok("abc123"));

        // Test JWT tokens
        let jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert_eq!(
            extract_bearer_token(&format!("Bearer {}", jwt_token)),
            Ok(jwt_token)
        );

        // Test error cases
        assert_eq!(
            extract_bearer_token("Basic abc123"),
            Err(AuthError::InvalidToken("Not a Bearer token".to_string()))
        );

        assert_eq!(
            extract_bearer_token(""),
            Err(AuthError::InvalidToken("Not a Bearer token".to_string()))
        );

        // Test token length validation
        let long_token = "a".repeat(4097);
        assert_eq!(
            extract_bearer_token(&format!("Bearer {}", long_token)),
            Err(AuthError::InvalidToken(
                "Token too long: 4097 characters (max: 4096)".to_string()
            ))
        );
    }

    #[test]
    fn test_extract_jwt_bearer_token() {
        // Test valid JWT tokens
        let jwt_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert_eq!(
            extract_jwt_bearer_token(&format!("Bearer {}", jwt_token)),
            Ok(jwt_token)
        );

        // Test invalid JWT format (simple token)
        assert_eq!(
            extract_jwt_bearer_token("Bearer abc123"),
            Err(AuthError::InvalidToken(
                "Invalid JWT format: expected 3 parts".to_string()
            ))
        );

        // Test invalid JWT format (wrong number of parts)
        assert_eq!(
            extract_jwt_bearer_token("Bearer part1.part2"),
            Err(AuthError::InvalidToken(
                "Invalid JWT format: expected 3 parts".to_string()
            ))
        );

        assert_eq!(
            extract_jwt_bearer_token("Bearer part1.part2.part3.part4"),
            Err(AuthError::InvalidToken(
                "Invalid JWT format: expected 3 parts".to_string()
            ))
        );
    }
}
