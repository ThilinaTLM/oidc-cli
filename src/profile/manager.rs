#![allow(dead_code)]

use std::path::Path;

use crate::config::{Config, Profile};
use crate::error::{OidcError, Result};
use crate::profile::storage::ProfileStorage;
use crate::profile::validation::{validate_profile_input, sanitize_input};

pub struct ProfileParams {
    pub name: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub scope: String,
    pub discovery_uri: Option<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
}

pub struct ProfileManager {
    config: Config,
}

impl ProfileManager {
    pub fn new() -> Result<Self> {
        let config = ProfileStorage::load_config()?;
        Ok(ProfileManager { config })
    }

    pub fn list_profiles(&self) -> Vec<&String> {
        self.config.list_profiles()
    }

    pub fn get_profile(&self, name: &str) -> Result<&Profile> {
        self.config.get_profile(name)
    }

    pub fn create_profile(&mut self, params: ProfileParams) -> Result<()> {
        let name = sanitize_input(&params.name);
        let client_id = sanitize_input(&params.client_id);
        let redirect_uri = sanitize_input(&params.redirect_uri);
        let scope = sanitize_input(&params.scope);
        
        let client_secret = params.client_secret.map(|s| sanitize_input(&s));
        let discovery_uri = params.discovery_uri.map(|s| sanitize_input(&s));
        let authorization_endpoint = params.authorization_endpoint.map(|s| sanitize_input(&s));
        let token_endpoint = params.token_endpoint.map(|s| sanitize_input(&s));

        if name.is_empty() {
            return Err(OidcError::Config("Profile name cannot be empty".to_string()));
        }

        validate_profile_input(
            &client_id,
            &redirect_uri,
            &scope,
            discovery_uri.as_deref(),
            authorization_endpoint.as_deref(),
            token_endpoint.as_deref(),
        )?;

        let profile = Profile {
            discovery_uri,
            client_id,
            client_secret,
            redirect_uri,
            scope,
            authorization_endpoint,
            token_endpoint,
        };

        self.config.add_profile(name, profile)?;
        self.save()?;
        Ok(())
    }

    pub fn update_profile(&mut self, params: ProfileParams) -> Result<()> {
        let name = sanitize_input(&params.name);
        let client_id = sanitize_input(&params.client_id);
        let redirect_uri = sanitize_input(&params.redirect_uri);
        let scope = sanitize_input(&params.scope);
        
        let client_secret = params.client_secret.map(|s| sanitize_input(&s));
        let discovery_uri = params.discovery_uri.map(|s| sanitize_input(&s));
        let authorization_endpoint = params.authorization_endpoint.map(|s| sanitize_input(&s));
        let token_endpoint = params.token_endpoint.map(|s| sanitize_input(&s));

        validate_profile_input(
            &client_id,
            &redirect_uri,
            &scope,
            discovery_uri.as_deref(),
            authorization_endpoint.as_deref(),
            token_endpoint.as_deref(),
        )?;

        let profile = Profile {
            discovery_uri,
            client_id,
            client_secret,
            redirect_uri,
            scope,
            authorization_endpoint,
            token_endpoint,
        };

        self.config.update_profile(name, profile)?;
        self.save()?;
        Ok(())
    }

    pub fn delete_profile(&mut self, name: &str) -> Result<()> {
        self.config.remove_profile(name)?;
        self.save()?;
        Ok(())
    }

    pub fn rename_profile(&mut self, old_name: &str, new_name: String) -> Result<()> {
        let new_name = sanitize_input(&new_name);
        
        if new_name.is_empty() {
            return Err(OidcError::Config("New profile name cannot be empty".to_string()));
        }
        
        self.config.rename_profile(old_name, new_name)?;
        self.save()?;
        Ok(())
    }

    pub fn export_profiles(&self, file_path: &Path, profile_names: Option<Vec<String>>) -> Result<()> {
        let export_config = if let Some(names) = profile_names {
            let mut filtered_config = Config::new();
            for name in names {
                let profile = self.config.get_profile(&name)?.clone();
                filtered_config.profiles.insert(name, profile);
            }
            filtered_config
        } else {
            self.config.clone()
        };

        ProfileStorage::export_config(&export_config, file_path)
    }

    pub fn import_profiles(&mut self, file_path: &Path, overwrite: bool) -> Result<Vec<String>> {
        let imported_config = ProfileStorage::import_config(file_path)?;
        let mut imported_names = Vec::new();

        for (name, profile) in imported_config.profiles {
            if self.config.profiles.contains_key(&name) && !overwrite {
                return Err(OidcError::ProfileExists(format!(
                    "Profile '{name}' already exists. Use --overwrite to replace it."
                )));
            }

            if overwrite {
                self.config.profiles.insert(name.clone(), profile);
            } else {
                self.config.add_profile(name.clone(), profile)?;
            }
            
            imported_names.push(name);
        }

        self.save()?;
        Ok(imported_names)
    }

    pub fn has_profiles(&self) -> bool {
        !self.config.profiles.is_empty()
    }

    pub fn get_single_profile(&self) -> Option<(&String, &Profile)> {
        if self.config.profiles.len() == 1 {
            self.config.profiles.iter().next()
        } else {
            None
        }
    }

    fn save(&self) -> Result<()> {
        ProfileStorage::save_config(&self.config)
    }
}

impl Clone for ProfileManager {
    fn clone(&self) -> Self {
        ProfileManager {
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile_manager() -> ProfileManager {
        ProfileManager {
            config: Config::new(),
        }
    }

    #[test]
    fn test_create_profile() {
        let mut manager = create_test_profile_manager();
        
        let result = manager.create_profile(ProfileParams {
            name: "test".to_string(),
            client_id: "test-client".to_string(),
            client_secret: Some("test-secret".to_string()),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scope: "openid profile email".to_string(),
            discovery_uri: Some("https://example.com/.well-known/openid-configuration".to_string()),
            authorization_endpoint: None,
            token_endpoint: None,
        });
        
        assert!(result.is_ok());
        assert!(manager.get_profile("test").is_ok());
    }

    #[test]
    fn test_create_duplicate_profile() {
        let mut manager = create_test_profile_manager();
        
        manager.create_profile(ProfileParams {
            name: "test".to_string(),
            client_id: "test-client".to_string(),
            client_secret: None,
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scope: "openid".to_string(),
            discovery_uri: Some("https://example.com/.well-known/openid-configuration".to_string()),
            authorization_endpoint: None,
            token_endpoint: None,
        }).unwrap();
        
        let result = manager.create_profile(ProfileParams {
            name: "test".to_string(),
            client_id: "test-client-2".to_string(),
            client_secret: None,
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scope: "openid".to_string(),
            discovery_uri: Some("https://example.com/.well-known/openid-configuration".to_string()),
            authorization_endpoint: None,
            token_endpoint: None,
        });
        
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_profile() {
        let mut manager = create_test_profile_manager();
        
        manager.create_profile(ProfileParams {
            name: "test".to_string(),
            client_id: "test-client".to_string(),
            client_secret: None,
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scope: "openid".to_string(),
            discovery_uri: Some("https://example.com/.well-known/openid-configuration".to_string()),
            authorization_endpoint: None,
            token_endpoint: None,
        }).unwrap();
        
        assert!(manager.delete_profile("test").is_ok());
        assert!(manager.get_profile("test").is_err());
    }

    #[test]
    fn test_rename_profile() {
        let mut manager = create_test_profile_manager();
        
        manager.create_profile(ProfileParams {
            name: "test".to_string(),
            client_id: "test-client".to_string(),
            client_secret: None,
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scope: "openid".to_string(),
            discovery_uri: Some("https://example.com/.well-known/openid-configuration".to_string()),
            authorization_endpoint: None,
            token_endpoint: None,
        }).unwrap();
        
        assert!(manager.rename_profile("test", "new-test".to_string()).is_ok());
        assert!(manager.get_profile("test").is_err());
        assert!(manager.get_profile("new-test").is_ok());
    }
}