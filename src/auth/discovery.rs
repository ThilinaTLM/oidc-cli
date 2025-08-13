#![allow(dead_code)]

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

use crate::error::{OidcError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryDocument {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub issuer: String,
    pub response_types_supported: Option<Vec<String>>,
    pub subject_types_supported: Option<Vec<String>>,
    pub id_token_signing_alg_values_supported: Option<Vec<String>>,
    pub scopes_supported: Option<Vec<String>>,
    pub token_endpoint_auth_methods_supported: Option<Vec<String>>,
    pub code_challenge_methods_supported: Option<Vec<String>>,
}

impl DiscoveryDocument {
    pub fn supports_pkce(&self) -> bool {
        self.code_challenge_methods_supported
            .as_ref()
            .is_some_and(|methods| methods.contains(&"S256".to_string()))
    }

    pub fn supports_authorization_code(&self) -> bool {
        self.response_types_supported
            .as_ref()
            .is_none_or(|types| types.contains(&"code".to_string()))
    }
}

pub async fn discover_endpoints(discovery_uri: &str) -> Result<DiscoveryDocument> {
    let url = Url::parse(discovery_uri)
        .map_err(|_| OidcError::Discovery(format!("Invalid discovery URI: {discovery_uri}")))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(OidcError::Discovery(format!(
            "Discovery request failed with status: {}",
            response.status()
        )));
    }

    let discovery_doc: DiscoveryDocument = response.json().await
        .map_err(|e| OidcError::Discovery(format!("Failed to parse discovery document: {e}")))?;

    validate_discovery_document(&discovery_doc)?;

    Ok(discovery_doc)
}

fn validate_discovery_document(doc: &DiscoveryDocument) -> Result<()> {
    if doc.authorization_endpoint.is_empty() {
        return Err(OidcError::Discovery(
            "Missing authorization_endpoint in discovery document".to_string()
        ));
    }

    if doc.token_endpoint.is_empty() {
        return Err(OidcError::Discovery(
            "Missing token_endpoint in discovery document".to_string()
        ));
    }

    if doc.issuer.is_empty() {
        return Err(OidcError::Discovery(
            "Missing issuer in discovery document".to_string()
        ));
    }

    Url::parse(&doc.authorization_endpoint)
        .map_err(|_| OidcError::Discovery("Invalid authorization_endpoint URL".to_string()))?;

    Url::parse(&doc.token_endpoint)
        .map_err(|_| OidcError::Discovery("Invalid token_endpoint URL".to_string()))?;

    if !doc.supports_authorization_code() {
        return Err(OidcError::Discovery(
            "Authorization code flow not supported".to_string()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_document_validation() {
        let doc = DiscoveryDocument {
            authorization_endpoint: "https://example.com/auth".to_string(),
            token_endpoint: "https://example.com/token".to_string(),
            userinfo_endpoint: None,
            jwks_uri: None,
            issuer: "https://example.com".to_string(),
            response_types_supported: Some(vec!["code".to_string()]),
            subject_types_supported: None,
            id_token_signing_alg_values_supported: None,
            scopes_supported: None,
            token_endpoint_auth_methods_supported: None,
            code_challenge_methods_supported: Some(vec!["S256".to_string()]),
        };

        assert!(validate_discovery_document(&doc).is_ok());
        assert!(doc.supports_pkce());
        assert!(doc.supports_authorization_code());
    }

    #[test]
    fn test_discovery_document_missing_endpoints() {
        let doc = DiscoveryDocument {
            authorization_endpoint: "".to_string(),
            token_endpoint: "https://example.com/token".to_string(),
            userinfo_endpoint: None,
            jwks_uri: None,
            issuer: "https://example.com".to_string(),
            response_types_supported: None,
            subject_types_supported: None,
            id_token_signing_alg_values_supported: None,
            scopes_supported: None,
            token_endpoint_auth_methods_supported: None,
            code_challenge_methods_supported: None,
        };

        assert!(validate_discovery_document(&doc).is_err());
    }
}