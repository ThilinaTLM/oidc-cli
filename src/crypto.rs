use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::error::Result;

pub struct PkceChallenge {
    pub verifier: String,
    pub challenge: String,
}

impl PkceChallenge {
    pub fn new() -> Result<Self> {
        let verifier = generate_code_verifier()?;
        let challenge = create_code_challenge(&verifier)?;
        
        Ok(PkceChallenge {
            verifier,
            challenge,
        })
    }
}

pub fn generate_code_verifier() -> Result<String> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; 32];
    rng.fill(&mut bytes[..]);
    
    let verifier = URL_SAFE_NO_PAD.encode(&bytes);
    
    if verifier.len() < 43 {
        return Err(crate::error::OidcError::Config(
            "Code verifier must be at least 43 characters".to_string(),
        ));
    }
    
    Ok(verifier)
}

pub fn create_code_challenge(verifier: &str) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let digest = hasher.finalize();
    
    Ok(URL_SAFE_NO_PAD.encode(digest))
}

pub fn generate_state() -> Result<String> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; 16];
    rng.fill(&mut bytes[..]);
    
    Ok(URL_SAFE_NO_PAD.encode(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_verifier_length() {
        let verifier = generate_code_verifier().unwrap();
        assert!(verifier.len() >= 43);
    }

    #[test]
    fn test_state_generation() {
        let state = generate_state().unwrap();
        assert!(!state.is_empty());
        assert_eq!(state.len(), 22);
    }

    #[test]
    fn test_pkce_challenge() {
        let pkce = PkceChallenge::new().unwrap();
        assert!(pkce.verifier.len() >= 43);
        assert!(!pkce.challenge.is_empty());
        assert_ne!(pkce.verifier, pkce.challenge);
    }

    #[test]
    fn test_code_challenge_deterministic() {
        let verifier = "test_verifier_with_sufficient_length_for_pkce_requirements";
        let challenge1 = create_code_challenge(verifier).unwrap();
        let challenge2 = create_code_challenge(verifier).unwrap();
        assert_eq!(challenge1, challenge2);
    }
}