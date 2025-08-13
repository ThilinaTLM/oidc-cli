use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

use crate::auth::{discover_endpoints, generate_state, PkceChallenge};
use crate::config::Profile;
use crate::error::{OidcError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub scope: Option<String>,
}

pub struct AuthorizationRequest {
    pub authorization_url: String,
    pub state: String,
    pub pkce_challenge: PkceChallenge,
}

#[derive(Clone)]
pub struct OAuthClient {
    client: Client,
    profile: Profile,
    authorization_endpoint: String,
    token_endpoint: String,
}

impl OAuthClient {
    pub async fn new(profile: Profile) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        let (authorization_endpoint, token_endpoint) =
            if let Some(ref discovery_uri) = profile.discovery_uri {
                let discovery_doc = discover_endpoints(discovery_uri).await?;
                (
                    discovery_doc.authorization_endpoint,
                    discovery_doc.token_endpoint,
                )
            } else {
                let auth_endpoint = profile.authorization_endpoint.as_ref().ok_or_else(|| {
                    OidcError::Config("Missing authorization endpoint".to_string())
                })?;
                let token_endpoint = profile
                    .token_endpoint
                    .as_ref()
                    .ok_or_else(|| OidcError::Config("Missing token endpoint".to_string()))?;
                (auth_endpoint.clone(), token_endpoint.clone())
            };

        Ok(OAuthClient {
            client,
            profile,
            authorization_endpoint,
            token_endpoint,
        })
    }

    pub fn create_authorization_request(&self) -> Result<AuthorizationRequest> {
        let pkce_challenge = PkceChallenge::new()?;
        let state = generate_state()?;

        let mut auth_url = Url::parse(&self.authorization_endpoint)?;

        {
            let mut query_pairs = auth_url.query_pairs_mut();
            query_pairs.append_pair("response_type", "code");
            query_pairs.append_pair("client_id", &self.profile.client_id);
            query_pairs.append_pair("redirect_uri", &self.profile.redirect_uri);
            query_pairs.append_pair("scope", &self.profile.scope);
            query_pairs.append_pair("state", &state);
            query_pairs.append_pair("code_challenge", &pkce_challenge.challenge);
            query_pairs.append_pair("code_challenge_method", "S256");
        }

        Ok(AuthorizationRequest {
            authorization_url: auth_url.to_string(),
            state,
            pkce_challenge,
        })
    }

    pub async fn exchange_code_for_tokens(
        &self,
        authorization_code: &str,
        state: &str,
        expected_state: &str,
        pkce_verifier: &str,
    ) -> Result<TokenResponse> {
        if state != expected_state {
            return Err(OidcError::StateMismatch);
        }

        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", authorization_code);
        params.insert("redirect_uri", &self.profile.redirect_uri);
        params.insert("client_id", &self.profile.client_id);
        params.insert("code_verifier", pkce_verifier);

        let mut request = self.client.post(&self.token_endpoint).form(&params);

        if let Some(ref client_secret) = self.profile.client_secret {
            request = request.basic_auth(&self.profile.client_id, Some(client_secret));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(OidcError::Auth(format!(
                "Token exchange failed with status {status}: {error_text}"
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| OidcError::Auth(format!("Failed to parse token response: {e}")))?;

        validate_token_response(&token_response)?;

        Ok(token_response)
    }
}

fn validate_token_response(response: &TokenResponse) -> Result<()> {
    if response.access_token.is_empty() {
        return Err(OidcError::InvalidTokenResponse);
    }

    if response.token_type.is_empty() {
        return Err(OidcError::InvalidTokenResponse);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Profile;

    fn create_test_profile() -> Profile {
        Profile {
            discovery_uri: None,
            client_id: "test-client".to_string(),
            client_secret: Some("test-secret".to_string()),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scope: "openid profile email".to_string(),
            authorization_endpoint: Some("https://example.com/auth".to_string()),
            token_endpoint: Some("https://example.com/token".to_string()),
        }
    }

    #[tokio::test]
    async fn test_oauth_client_creation() {
        let profile = create_test_profile();
        let client = OAuthClient::new(profile).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_authorization_request_creation() {
        let profile = create_test_profile();
        let client = OAuthClient::new(profile).await.unwrap();
        let auth_request = client.create_authorization_request();
        assert!(auth_request.is_ok());

        let request = auth_request.unwrap();
        assert!(request.authorization_url.contains("code_challenge"));
        assert!(request.authorization_url.contains("state"));
        assert!(!request.state.is_empty());
    }

    #[test]
    fn test_token_response_validation() {
        let valid_response = TokenResponse {
            access_token: "test-token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: None,
            id_token: None,
            scope: None,
        };
        assert!(validate_token_response(&valid_response).is_ok());

        let invalid_response = TokenResponse {
            access_token: "".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: None,
            id_token: None,
            scope: None,
        };
        assert!(validate_token_response(&invalid_response).is_err());
    }
}
