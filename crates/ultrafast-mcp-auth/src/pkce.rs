use crate::{error::AuthError, types::PkceParams};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use rand_distr::Alphanumeric;
use sha2::{Digest, Sha256};

/// Generate PKCE parameters for OAuth 2.1 authorization code flow
pub fn generate_pkce_params() -> Result<PkceParams, AuthError> {
    // Generate code verifier (43-128 characters, URL-safe)
    let code_verifier: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();

    // Generate code challenge using S256 method
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let challenge_bytes = hasher.finalize();
    let code_challenge = URL_SAFE_NO_PAD.encode(challenge_bytes);

    Ok(PkceParams {
        code_verifier,
        code_challenge,
        code_challenge_method: "S256".to_string(),
    })
}

/// Generate a cryptographically secure state parameter
pub fn generate_state() -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

/// Generate a secure session ID
pub fn generate_session_id() -> String {
    let bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
    URL_SAFE_NO_PAD.encode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pkce_params() {
        let params = generate_pkce_params().unwrap();
        assert_eq!(params.code_verifier.len(), 128);
        assert_eq!(params.code_challenge_method, "S256");
        assert!(!params.code_challenge.is_empty());

        // Verify code challenge is different from verifier
        assert_ne!(params.code_verifier, params.code_challenge);
    }

    #[test]
    fn test_generate_state() {
        let state1 = generate_state();
        let state2 = generate_state();

        assert_eq!(state1.len(), 32);
        assert_eq!(state2.len(), 32);
        assert_ne!(state1, state2); // Should be different
    }

    #[test]
    fn test_generate_session_id() {
        let session1 = generate_session_id();
        let session2 = generate_session_id();

        assert!(!session1.is_empty());
        assert!(!session2.is_empty());
        assert_ne!(session1, session2); // Should be different
    }
}
