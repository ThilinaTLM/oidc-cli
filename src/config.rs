use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;

use crate::error::{OidcError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub discovery_uri: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub scope: String,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
}

impl Profile {
    pub fn validate(&self) -> Result<()> {
        if self.client_id.is_empty() {
            return Err(OidcError::MissingField("client_id".to_string()));
        }

        if self.redirect_uri.is_empty() {
            return Err(OidcError::MissingField("redirect_uri".to_string()));
        }

        if self.scope.is_empty() {
            return Err(OidcError::MissingField("scope".to_string()));
        }

        Url::parse(&self.redirect_uri)
            .map_err(|_| OidcError::InvalidRedirectUri(self.redirect_uri.clone()))?;

        if let Some(ref discovery_uri) = self.discovery_uri {
            Url::parse(discovery_uri)
                .map_err(|_| OidcError::Config(format!("Invalid discovery URI: {}", discovery_uri)))?;
        }

        if let Some(ref auth_endpoint) = self.authorization_endpoint {
            Url::parse(auth_endpoint)
                .map_err(|_| OidcError::Config(format!("Invalid authorization endpoint: {}", auth_endpoint)))?;
        }

        if let Some(ref token_endpoint) = self.token_endpoint {
            Url::parse(token_endpoint)
                .map_err(|_| OidcError::Config(format!("Invalid token endpoint: {}", token_endpoint)))?;
        }

        if self.discovery_uri.is_none() 
            && (self.authorization_endpoint.is_none() || self.token_endpoint.is_none()) {
            return Err(OidcError::Config(
                "Either discovery_uri or both authorization_endpoint and token_endpoint must be provided".to_string()
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub profiles: HashMap<String, Profile>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            profiles: HashMap::new(),
        }
    }

    pub fn add_profile(&mut self, name: String, profile: Profile) -> Result<()> {
        profile.validate()?;
        
        if self.profiles.contains_key(&name) {
            return Err(OidcError::ProfileExists(name));
        }
        
        self.profiles.insert(name, profile);
        Ok(())
    }

    pub fn get_profile(&self, name: &str) -> Result<&Profile> {
        self.profiles.get(name)
            .ok_or_else(|| OidcError::ProfileNotFound(name.to_string()))
    }

    pub fn remove_profile(&mut self, name: &str) -> Result<Profile> {
        self.profiles.remove(name)
            .ok_or_else(|| OidcError::ProfileNotFound(name.to_string()))
    }

    pub fn update_profile(&mut self, name: String, profile: Profile) -> Result<()> {
        profile.validate()?;
        
        if !self.profiles.contains_key(&name) {
            return Err(OidcError::ProfileNotFound(name));
        }
        
        self.profiles.insert(name, profile);
        Ok(())
    }

    pub fn rename_profile(&mut self, old_name: &str, new_name: String) -> Result<()> {
        if self.profiles.contains_key(&new_name) {
            return Err(OidcError::ProfileExists(new_name));
        }
        
        let profile = self.remove_profile(old_name)?;
        self.profiles.insert(new_name, profile);
        Ok(())
    }

    pub fn list_profiles(&self) -> Vec<&String> {
        self.profiles.keys().collect()
    }
}

pub fn get_config_dir() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|mut path| {
            path.push("oidc-cli");
            path
        })
        .ok_or_else(|| OidcError::Config("Could not determine config directory".to_string()))
}

pub fn get_config_file_path() -> Result<PathBuf> {
    let mut path = get_config_dir()?;
    path.push("profiles.json");
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile() -> Profile {
        Profile {
            discovery_uri: Some("https://example.com/.well-known/openid-configuration".to_string()),
            client_id: "test-client".to_string(),
            client_secret: Some("test-secret".to_string()),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scope: "openid profile email".to_string(),
            authorization_endpoint: None,
            token_endpoint: None,
        }
    }

    #[test]
    fn test_profile_validation() {
        let profile = create_test_profile();
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_profile_validation_missing_client_id() {
        let mut profile = create_test_profile();
        profile.client_id = "".to_string();
        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_profile_validation_invalid_redirect_uri() {
        let mut profile = create_test_profile();
        profile.redirect_uri = "invalid-uri".to_string();
        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_config_add_profile() {
        let mut config = Config::new();
        let profile = create_test_profile();
        
        assert!(config.add_profile("test".to_string(), profile).is_ok());
        assert!(config.get_profile("test").is_ok());
    }

    #[test]
    fn test_config_duplicate_profile() {
        let mut config = Config::new();
        let profile = create_test_profile();
        
        config.add_profile("test".to_string(), profile.clone()).unwrap();
        assert!(config.add_profile("test".to_string(), profile).is_err());
    }
}