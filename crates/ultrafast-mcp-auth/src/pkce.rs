use crate::error::AuthError;
use crate::types::PkceParams;
use ultrafast_mcp_core::utils::generate_secure_random;
use sha2::Digest;
use base64::Engine;

/// Generate PKCE (Proof Key for Code Exchange) parameters
pub fn generate_pkce_params() -> Result<PkceParams, AuthError> {
    // Generate code verifier (43-128 characters)
    let code_verifier = generate_secure_random(128);
    
    // Generate code challenge using SHA256
    let digest = sha2::Sha256::digest(code_verifier.as_bytes());
    let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(digest);

    Ok(PkceParams {
        code_verifier,
        code_challenge,
        code_challenge_method: "S256".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ultrafast_mcp_core::utils::{generate_session_id, generate_state};

    #[test]
    fn test_generate_pkce_params() {
        let params = generate_pkce_params().unwrap();
        assert!(!params.code_verifier.is_empty());
        assert!(!params.code_challenge.is_empty());
        assert_eq!(params.code_challenge_method, "S256");
        assert_eq!(params.code_verifier.len(), 128);
    }

    #[test]
    fn test_generate_state() {
        let state1 = generate_state();
        let state2 = generate_state();
        assert_ne!(state1, state2);
        assert_eq!(state1.len(), 32);
        assert!(state1.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_session_id() {
        let session1 = generate_session_id();
        let session2 = generate_session_id();
        assert_ne!(session1, session2);
        assert_eq!(session1.len(), 36); // UUID format
        assert!(session1.contains('-'));
    }

    #[test]
    fn test_pkce_params_uniqueness() {
        let params1 = generate_pkce_params().unwrap();
        let params2 = generate_pkce_params().unwrap();
        assert_ne!(params1.code_verifier, params2.code_verifier);
        assert_ne!(params1.code_challenge, params2.code_challenge);
    }
}
