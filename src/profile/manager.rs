#![allow(dead_code)]

use std::path::Path;

use crate::config::{Config, Profile};
use crate::error::{OidcError, Result};
use crate::profile::storage::ProfileStorage;
use crate::profile::validation::{validate_profile_input, sanitize_input};

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

    pub fn create_profile(
        &mut self,
        name: String,
        client_id: String,
        client_secret: Option<String>,
        redirect_uri: String,
        scope: String,
        discovery_uri: Option<String>,
        authorization_endpoint: Option<String>,
        token_endpoint: Option<String>,
    ) -> Result<()> {
        let name = sanitize_input(&name);
        let client_id = sanitize_input(&client_id);
        let redirect_uri = sanitize_input(&redirect_uri);
        let scope = sanitize_input(&scope);
        
        let client_secret = client_secret.map(|s| sanitize_input(&s));
        let discovery_uri = discovery_uri.map(|s| sanitize_input(&s));
        let authorization_endpoint = authorization_endpoint.map(|s| sanitize_input(&s));
        let token_endpoint = token_endpoint.map(|s| sanitize_input(&s));

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

    pub fn update_profile(
        &mut self,
        name: String,
        client_id: String,
        client_secret: Option<String>,
        redirect_uri: String,
        scope: String,
        discovery_uri: Option<String>,
        authorization_endpoint: Option<String>,
        token_endpoint: Option<String>,
    ) -> Result<()> {
        let name = sanitize_input(&name);
        let client_id = sanitize_input(&client_id);
        let redirect_uri = sanitize_input(&redirect_uri);
        let scope = sanitize_input(&scope);
        
        let client_secret = client_secret.map(|s| sanitize_input(&s));
        let discovery_uri = discovery_uri.map(|s| sanitize_input(&s));
        let authorization_endpoint = authorization_endpoint.map(|s| sanitize_input(&s));
        let token_endpoint = token_endpoint.map(|s| sanitize_input(&s));

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
                    "Profile '{}' already exists. Use --overwrite to replace it.",
                    name
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
        
        let result = manager.create_profile(
            "test".to_string(),
            "test-client".to_string(),
            Some("test-secret".to_string()),
            "http://localhost:8080/callback".to_string(),
            "openid profile email".to_string(),
            Some("https://example.com/.well-known/openid-configuration".to_string()),
            None,
            None,
        );
        
        assert!(result.is_ok());
        assert!(manager.get_profile("test").is_ok());
    }

    #[test]
    fn test_create_duplicate_profile() {
        let mut manager = create_test_profile_manager();
        
        manager.create_profile(
            "test".to_string(),
            "test-client".to_string(),
            None,
            "http://localhost:8080/callback".to_string(),
            "openid".to_string(),
            Some("https://example.com/.well-known/openid-configuration".to_string()),
            None,
            None,
        ).unwrap();
        
        let result = manager.create_profile(
            "test".to_string(),
            "test-client-2".to_string(),
            None,
            "http://localhost:8080/callback".to_string(),
            "openid".to_string(),
            Some("https://example.com/.well-known/openid-configuration".to_string()),
            None,
            None,
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_profile() {
        let mut manager = create_test_profile_manager();
        
        manager.create_profile(
            "test".to_string(),
            "test-client".to_string(),
            None,
            "http://localhost:8080/callback".to_string(),
            "openid".to_string(),
            Some("https://example.com/.well-known/openid-configuration".to_string()),
            None,
            None,
        ).unwrap();
        
        assert!(manager.delete_profile("test").is_ok());
        assert!(manager.get_profile("test").is_err());
    }

    #[test]
    fn test_rename_profile() {
        let mut manager = create_test_profile_manager();
        
        manager.create_profile(
            "test".to_string(),
            "test-client".to_string(),
            None,
            "http://localhost:8080/callback".to_string(),
            "openid".to_string(),
            Some("https://example.com/.well-known/openid-configuration".to_string()),
            None,
            None,
        ).unwrap();
        
        assert!(manager.rename_profile("test", "new-test".to_string()).is_ok());
        assert!(manager.get_profile("test").is_err());
        assert!(manager.get_profile("new-test").is_ok());
    }
}