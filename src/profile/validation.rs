use url::Url;
use crate::config::Profile;
use crate::error::{OidcError, Result};

pub fn validate_profile_input(
    client_id: &str,
    redirect_uri: &str,
    scope: &str,
    discovery_uri: Option<&str>,
    authorization_endpoint: Option<&str>,
    token_endpoint: Option<&str>,
) -> Result<()> {
    validate_client_id(client_id)?;
    validate_redirect_uri(redirect_uri)?;
    validate_scope(scope)?;
    
    if let Some(uri) = discovery_uri {
        validate_discovery_uri(uri)?;
    }
    
    if let Some(endpoint) = authorization_endpoint {
        validate_endpoint_url(endpoint, "authorization endpoint")?;
    }
    
    if let Some(endpoint) = token_endpoint {
        validate_endpoint_url(endpoint, "token endpoint")?;
    }
    
    validate_endpoint_configuration(discovery_uri, authorization_endpoint, token_endpoint)?;
    
    Ok(())
}

pub fn validate_client_id(client_id: &str) -> Result<()> {
    if client_id.is_empty() {
        return Err(OidcError::Config("Client ID cannot be empty".to_string()));
    }
    
    if client_id.trim() != client_id {
        return Err(OidcError::Config("Client ID cannot have leading or trailing whitespace".to_string()));
    }
    
    if client_id.len() > 255 {
        return Err(OidcError::Config("Client ID cannot exceed 255 characters".to_string()));
    }
    
    Ok(())
}

pub fn validate_redirect_uri(redirect_uri: &str) -> Result<()> {
    if redirect_uri.is_empty() {
        return Err(OidcError::Config("Redirect URI cannot be empty".to_string()));
    }
    
    let url = Url::parse(redirect_uri)
        .map_err(|_| OidcError::InvalidRedirectUri(redirect_uri.to_string()))?;
    
    match url.scheme() {
        "http" | "https" => {
            if url.host_str().is_none() {
                return Err(OidcError::InvalidRedirectUri("Redirect URI must have a valid host".to_string()));
            }
        }
        _ => {
            return Err(OidcError::InvalidRedirectUri("Redirect URI must use http or https scheme".to_string()));
        }
    }
    
    Ok(())
}

pub fn validate_scope(scope: &str) -> Result<()> {
    if scope.is_empty() {
        return Err(OidcError::Config("Scope cannot be empty".to_string()));
    }
    
    let scopes: Vec<&str> = scope.split_whitespace().collect();
    if scopes.is_empty() {
        return Err(OidcError::Config("Scope must contain at least one valid scope value".to_string()));
    }
    
    for scope_value in scopes {
        if scope_value.is_empty() {
            return Err(OidcError::Config("Individual scope values cannot be empty".to_string()));
        }
        
        if !scope_value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':') {
            return Err(OidcError::Config(format!("Invalid scope value '{}': must contain only alphanumeric characters, underscores, hyphens, dots, or colons", scope_value)));
        }
    }
    
    Ok(())
}

pub fn validate_discovery_uri(discovery_uri: &str) -> Result<()> {
    if discovery_uri.is_empty() {
        return Err(OidcError::Config("Discovery URI cannot be empty".to_string()));
    }
    
    let url = Url::parse(discovery_uri)
        .map_err(|_| OidcError::Config(format!("Invalid discovery URI: {}", discovery_uri)))?;
    
    if url.scheme() != "https" {
        return Err(OidcError::Config("Discovery URI must use HTTPS".to_string()));
    }
    
    if url.host_str().is_none() {
        return Err(OidcError::Config("Discovery URI must have a valid host".to_string()));
    }
    
    Ok(())
}

pub fn validate_endpoint_url(endpoint: &str, endpoint_type: &str) -> Result<()> {
    if endpoint.is_empty() {
        return Err(OidcError::Config(format!("{} cannot be empty", endpoint_type)));
    }
    
    let url = Url::parse(endpoint)
        .map_err(|_| OidcError::Config(format!("Invalid {} URL: {}", endpoint_type, endpoint)))?;
    
    if url.scheme() != "https" {
        return Err(OidcError::Config(format!("{} must use HTTPS", endpoint_type)));
    }
    
    if url.host_str().is_none() {
        return Err(OidcError::Config(format!("{} must have a valid host", endpoint_type)));
    }
    
    Ok(())
}

pub fn validate_endpoint_configuration(
    discovery_uri: Option<&str>,
    authorization_endpoint: Option<&str>,
    token_endpoint: Option<&str>,
) -> Result<()> {
    if discovery_uri.is_none() {
        if authorization_endpoint.is_none() || token_endpoint.is_none() {
            return Err(OidcError::Config(
                "Either discovery URI or both authorization and token endpoints must be provided".to_string()
            ));
        }
    }
    
    Ok(())
}

pub fn sanitize_input(input: &str) -> String {
    input.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_client_id() {
        assert!(validate_client_id("valid-client-id").is_ok());
        assert!(validate_client_id("").is_err());
        assert!(validate_client_id(" invalid ").is_err());
        assert!(validate_client_id(&"x".repeat(256)).is_err());
    }

    #[test]
    fn test_validate_redirect_uri() {
        assert!(validate_redirect_uri("http://localhost:8080/callback").is_ok());
        assert!(validate_redirect_uri("https://example.com/callback").is_ok());
        assert!(validate_redirect_uri("").is_err());
        assert!(validate_redirect_uri("invalid-uri").is_err());
        assert!(validate_redirect_uri("ftp://example.com/callback").is_err());
    }

    #[test]
    fn test_validate_scope() {
        assert!(validate_scope("openid profile email").is_ok());
        assert!(validate_scope("openid").is_ok());
        assert!(validate_scope("").is_err());
        assert!(validate_scope("   ").is_err());
        assert!(validate_scope("invalid scope!").is_err());
    }

    #[test]
    fn test_validate_discovery_uri() {
        assert!(validate_discovery_uri("https://example.com/.well-known/openid-configuration").is_ok());
        assert!(validate_discovery_uri("").is_err());
        assert!(validate_discovery_uri("http://example.com/.well-known/openid-configuration").is_err());
        assert!(validate_discovery_uri("invalid-uri").is_err());
    }

    #[test]
    fn test_validate_endpoint_configuration() {
        assert!(validate_endpoint_configuration(
            Some("https://example.com/.well-known/openid-configuration"),
            None,
            None
        ).is_ok());
        
        assert!(validate_endpoint_configuration(
            None,
            Some("https://example.com/auth"),
            Some("https://example.com/token")
        ).is_ok());
        
        assert!(validate_endpoint_configuration(
            None,
            Some("https://example.com/auth"),
            None
        ).is_err());
    }
}