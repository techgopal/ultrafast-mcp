use crate::{error::AuthError, types::{OAuthConfig, TokenResponse, PkceParams, AuthorizationServerMetadata, ClientRegistrationRequest, ClientRegistrationResponse}};
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
};
use reqwest::Client;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use url::Url;
use rand::Rng;
use sha2::{Digest, Sha256};

/// OAuth 2.1 client for server discovery and registration
pub struct OAuthClient {
    http_client: Client,
    client_id: String,
    #[allow(dead_code)]
    client_secret: String,
    auth_url: String,
}

impl OAuthClient {
    pub fn new(client_id: String, client_secret: String, auth_url: String) -> Self {
        Self {
            http_client: Client::new(),
            client_id,
            client_secret,
            auth_url,
        }
    }
    
    pub fn from_config(config: OAuthConfig) -> Self {
        Self::new(config.client_id, config.client_secret, config.auth_url)
    }
    
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
    
    pub fn auth_url(&self) -> &str {
        &self.auth_url
    }
    
    pub async fn get_authorization_url(&self, state: String) -> Result<String, AuthError> {
        Ok(format!("{}?client_id={}&state={}", self.auth_url, self.client_id, state))
    }
    
    pub async fn get_authorization_url_with_pkce(&self, state: String, pkce: PkceParams) -> Result<String, AuthError> {
        Ok(format!("{}?client_id={}&state={}&code_challenge={}&code_challenge_method={}", 
            self.auth_url, self.client_id, state, pkce.code_challenge, pkce.code_challenge_method))
    }
    
    /// Discover authorization server metadata (RFC 8414)
    pub async fn discover_server_metadata(&self, issuer: &str) -> Result<AuthorizationServerMetadata, AuthError> {
        let discovery_url = if issuer.ends_with('/') {
            format!("{}/.well-known/authorization-server", issuer.trim_end_matches('/'))
        } else {
            format!("{}/.well-known/authorization-server", issuer)
        };
        
        let response = self.http_client
            .get(&discovery_url)
            .header("Accept", "application/json")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(AuthError::AuthorizationServerError {
                error: format!("Discovery failed with status: {}", response.status()),
            });
        }
        
        let metadata: AuthorizationServerMetadata = response.json().await?;
        Ok(metadata)
    }
    
    /// Register a dynamic client (RFC 7591)
    pub async fn register_client(
        &self,
        registration_endpoint: &str,
        request: ClientRegistrationRequest,
    ) -> Result<ClientRegistrationResponse, AuthError> {
        let response = self.http_client
            .post(registration_endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(AuthError::AuthorizationServerError {
                error: format!("Client registration failed: {}", error_body),
            });
        }
        
        let registration_response: ClientRegistrationResponse = response.json().await?;
        Ok(registration_response)
    }
    
    /// Build authorization URL with PKCE
    pub fn build_authorization_url(
        &self,
        authorization_endpoint: &str,
        client_id: &str,
        redirect_uri: &str,
        scopes: &[String],
        state: &str,
        code_challenge: &str,
        code_challenge_method: &str,
        audience: Option<&str>,
    ) -> Result<String, AuthError> {
        let mut url = Url::parse(authorization_endpoint)?;
        let scope_str = scopes.join(" ");
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", client_id)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("scope", &scope_str)
            .append_pair("state", state)
            .append_pair("code_challenge", code_challenge)
            .append_pair("code_challenge_method", code_challenge_method);
        if let Some(aud) = audience {
            url.query_pairs_mut().append_pair("audience", aud);
        }
        Ok(url.to_string())
    }
    
    /// Exchange authorization code for access token
    pub async fn exchange_code_for_token(
        &self,
        token_endpoint: &str,
        client_id: &str,
        client_secret: Option<&str>,
        redirect_uri: &str,
        authorization_code: &str,
        code_verifier: &str,
    ) -> Result<TokenResponse, AuthError> {
        use std::collections::HashMap;
        
        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("client_id", client_id);
        params.insert("redirect_uri", redirect_uri);
        params.insert("code", authorization_code);
        params.insert("code_verifier", code_verifier);
        
        let mut request_builder = self.http_client
            .post(token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json");
        
        // Add client authentication
        if let Some(secret) = client_secret {
            request_builder = request_builder.basic_auth(client_id, Some(secret));
        }
        
        let response = request_builder
            .form(&params)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(AuthError::TokenExchangeError {
                error: format!("Token exchange failed: {}", error_body),
            });
        }
        
        let token_response: TokenResponse = response.json().await?;
        Ok(token_response)
    }
    
    /// Refresh an access token using a refresh token
    pub async fn refresh_token(
        &self,
        token_endpoint: &str,
        client_id: &str,
        client_secret: Option<&str>,
        refresh_token: &str,
    ) -> Result<TokenResponse, AuthError> {
        use std::collections::HashMap;
        
        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", refresh_token);
        params.insert("client_id", client_id);
        
        let mut request_builder = self.http_client
            .post(token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json");
        
        // Add client authentication
        if let Some(secret) = client_secret {
            request_builder = request_builder.basic_auth(client_id, Some(secret));
        }
        
        let response = request_builder
            .form(&params)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(AuthError::TokenExchangeError {
                error: format!("Token refresh failed: {}", error_body),
            });
        }
        
        let token_response: TokenResponse = response.json().await?;
        Ok(token_response)
    }
    
    /// Validate token with introspection endpoint (RFC 7662)
    pub async fn introspect_token(
        &self,
        introspection_endpoint: &str,
        token: &str,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> Result<serde_json::Value, AuthError> {
        use std::collections::HashMap;
        
        let mut params = HashMap::new();
        params.insert("token", token);
        
        let mut request_builder = self.http_client
            .post(introspection_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json");
        
        // Add client authentication
        if let Some(secret) = client_secret {
            request_builder = request_builder.basic_auth(client_id, Some(secret));
        }
        
        let response = request_builder
            .form(&params)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(AuthError::TokenValidationError {
                reason: format!("Token introspection failed: {}", error_body),
            });
        }
        
        let introspection_response: serde_json::Value = response.json().await?;
        Ok(introspection_response)
    }
}

impl Default for OAuthClient {
    fn default() -> Self {
        Self::new(
            "default_client_id".to_string(),
            "default_client_secret".to_string(),
            "https://example.com/oauth/authorize".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_authorization_url() {
        let client = OAuthClient::new(
            "client123".to_string(),
            "secret456".to_string(),
            "https://auth.example.com/oauth".to_string(),
        );
        let url = client.build_authorization_url(
            "https://auth.example.com/authorize",
            "client123",
            "https://app.example.com/callback",
            &["read".to_string(), "write".to_string()],
            "state123",
            "challenge123",
            "S256",
            Some("https://api.example.com"),
        ).unwrap();
        println!("Generated URL: {}", url);
        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=client123"));
        assert!(url.contains("scope=read+write"));
        assert!(url.contains("state=state123"));
        assert!(url.contains("code_challenge=challenge123"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("audience=https%3A%2F%2Fapi.example.com"));
    }
}
