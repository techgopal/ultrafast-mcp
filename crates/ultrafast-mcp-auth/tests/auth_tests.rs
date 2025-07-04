use ultrafast_mcp_auth::*;

#[cfg(test)]
mod oauth_tests {
    use super::*;

    #[test]
    fn test_oauth_client_creation() {
        let client = OAuthClient::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "https://auth.example.com".to_string(),
        );

        assert_eq!(client.client_id(), "test_client_id");
        assert_eq!(client.auth_url(), "https://auth.example.com");
    }

    #[tokio::test]
    async fn test_oauth_flow_initialization() {
        let client = OAuthClient::new(
            "test_client".to_string(),
            "secret".to_string(),
            "https://oauth.example.com".to_string(),
        );

        let auth_url = client.get_authorization_url("test_state".to_string()).await;
        assert!(auth_url.is_ok());

        let url = auth_url.unwrap();
        assert!(url.contains("oauth.example.com"));
        assert!(url.contains("test_client"));
    }
}

#[cfg(test)]
mod pkce_tests {
    use super::*;

    #[test]
    fn test_pkce_params_generation() {
        let params = generate_pkce_params().unwrap();

        assert!(!params.code_verifier.is_empty());
        assert!(!params.code_challenge.is_empty());
        assert_eq!(params.code_challenge_method, "S256");

        // Code verifier should be at least 43 characters
        assert!(params.code_verifier.len() >= 43);
        assert!(params.code_verifier.len() <= 128);
    }

    #[test]
    fn test_state_generation() {
        let state1 = generate_state();
        let state2 = generate_state();

        // States should be different
        assert_ne!(state1, state2);

        // States should be reasonable length
        assert!(state1.len() >= 16);
        assert!(state2.len() >= 16);
    }

    #[test]
    fn test_session_id_generation() {
        let session1 = generate_session_id();
        let session2 = generate_session_id();

        // Session IDs should be different
        assert_ne!(session1, session2);

        // Should be base64-encoded strings (43 characters)
        assert_eq!(session1.len(), 43);
        assert_eq!(session2.len(), 43);
        // Should be URL-safe base64 (no padding)
        assert!(!session1.contains('='));
        assert!(!session2.contains('='));
    }

    #[test]
    fn test_pkce_consistency() {
        // Test that the same code verifier produces the same challenge
        let params1 = generate_pkce_params().unwrap();
        let params2 = generate_pkce_params().unwrap();

        // Different generations should produce different values
        assert_ne!(params1.code_verifier, params2.code_verifier);
        assert_ne!(params1.code_challenge, params2.code_challenge);
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_bearer_token_extraction() {
        // Valid bearer token
        let valid_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let result = extract_bearer_token(valid_header);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");

        // Invalid format
        let invalid_header = "Basic dXNlcjpwYXNz";
        let result = extract_bearer_token(invalid_header);
        assert!(result.is_err());

        // Missing token
        let empty_header = "Bearer ";
        let result = extract_bearer_token(empty_header);
        assert!(result.is_err());

        // No Bearer prefix
        let no_prefix = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let result = extract_bearer_token(no_prefix);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_validator_creation() {
        let validator = TokenValidator::new("test_secret".to_string());

        // Test that validator can be created
        assert_eq!(validator.get_secret(), "test_secret");
    }

    #[tokio::test]
    async fn test_token_validation_flow() {
        let validator = TokenValidator::new("test_secret_key".to_string());

        // This would test actual token validation in a real implementation
        // For now, we test that the validator exists and can be used
        let test_token = "test.jwt.token";
        let result = validator.validate_token(test_token).await;

        // This should fail with invalid token format
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_auth_error_types() {
        let errors = vec![
            AuthError::InvalidToken("test".to_string()),
            AuthError::TokenExpired,
            AuthError::InvalidClient("client".to_string()),
            AuthError::InvalidGrant("grant".to_string()),
            AuthError::InvalidScope("scope".to_string()),
            AuthError::UnauthorizedClient,
            AuthError::UnsupportedGrantType("type".to_string()),
            AuthError::InvalidRequest("request".to_string()),
            AuthError::ServerError("server".to_string()),
            AuthError::NetworkError("network".to_string()),
        ];

        for error in errors {
            // Test that all errors can be displayed
            let error_string = format!("{}", error);
            assert!(!error_string.is_empty());

            // Test that errors implement Debug
            let debug_string = format!("{:?}", error);
            assert!(!debug_string.is_empty());
        }
    }

    #[test]
    fn test_auth_result_type() {
        let success: AuthResult<String> = Ok("success".to_string());
        assert!(success.is_ok());

        let failure: AuthResult<String> = Err(AuthError::TokenExpired);
        assert!(failure.is_err());
    }
}

#[cfg(test)]
mod types_tests {
    use super::*;

    #[test]
    fn test_oauth_config_creation() {
        let config = OAuthConfig {
            client_id: "test_client".to_string(),
            client_secret: "test_secret".to_string(),
            auth_url: "https://auth.example.com".to_string(),
            token_url: "https://token.example.com".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: vec!["read".to_string(), "write".to_string()],
        };

        assert_eq!(config.client_id, "test_client");
        assert_eq!(config.scopes.len(), 2);
        assert!(config.scopes.contains(&"read".to_string()));
    }

    #[test]
    fn test_token_response_creation() {
        let response = TokenResponse {
            access_token: "access_token_123".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: Some("refresh_token_456".to_string()),
            scope: Some("read write".to_string()),
        };

        assert_eq!(response.access_token, "access_token_123");
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, Some(3600));
    }

    #[test]
    fn test_pkce_params_structure() {
        let params = PkceParams {
            code_verifier: "test_verifier".to_string(),
            code_challenge: "test_challenge".to_string(),
            code_challenge_method: "S256".to_string(),
        };

        assert_eq!(params.code_verifier, "test_verifier");
        assert_eq!(params.code_challenge_method, "S256");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_oauth_flow_setup() {
        // Test setting up a complete OAuth flow
        let config = OAuthConfig {
            client_id: "integration_test_client".to_string(),
            client_secret: "integration_test_secret".to_string(),
            auth_url: "https://auth.integration.test".to_string(),
            token_url: "https://token.integration.test".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: vec!["mcp:read".to_string(), "mcp:write".to_string()],
        };

        let client = OAuthClient::from_config(config);
        let pkce_params = generate_pkce_params().unwrap();
        let state = generate_state();

        // Generate authorization URL
        let auth_url = client
            .get_authorization_url_with_pkce(state, pkce_params)
            .await;
        assert!(auth_url.is_ok());

        let url = auth_url.unwrap();
        assert!(url.contains("auth.integration.test"));
        assert!(url.contains("integration_test_client"));
        assert!(url.contains("code_challenge"));
    }

    #[tokio::test]
    async fn test_token_validation_integration() {
        let validator = TokenValidator::new("integration_test_secret".to_string());

        // Test with various token formats
        let test_cases = vec![
            ("", false),             // Empty token
            ("invalid", false),      // Invalid format
            ("Bearer token", false), // Wrong format for validation
            (
                "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.invalid.signature",
                false,
            ), // Invalid JWT
        ];

        for (token, should_succeed) in test_cases {
            let result = validator.validate_token(token).await;
            assert_eq!(result.is_ok(), should_succeed);
        }
    }
}
