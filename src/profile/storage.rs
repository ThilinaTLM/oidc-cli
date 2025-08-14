use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::config::{get_config_dir_with_override, get_config_file_path_with_override, Config};
use crate::error::{OidcError, Result};

pub struct ProfileStorage;

impl ProfileStorage {
    pub fn load_config_with_override(override_dir: Option<PathBuf>) -> Result<Config> {
        let config_path = get_config_file_path_with_override(override_dir)?;

        if !config_path.exists() {
            return Ok(Config::new());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| OidcError::Profile(format!("Failed to read config file: {e}")))?;

        if content.trim().is_empty() {
            return Ok(Config::new());
        }

        let config: Config = serde_json::from_str(&content)
            .map_err(|e| OidcError::Profile(format!("Failed to parse config file: {e}")))?;

        for (name, profile) in &config.profiles {
            profile
                .validate()
                .map_err(|e| OidcError::Profile(format!("Invalid profile '{name}': {e}")))?;
        }

        Ok(config)
    }

    pub fn save_config_with_override(config: &Config, override_dir: Option<PathBuf>) -> Result<()> {
        let config_dir = get_config_dir_with_override(override_dir.clone())?;
        let config_path = get_config_file_path_with_override(override_dir)?;

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(|e| {
                OidcError::Profile(format!("Failed to create config directory: {e}"))
            })?;
        }

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| OidcError::Profile(format!("Failed to serialize config: {e}")))?;

        fs::write(&config_path, json)
            .map_err(|e| OidcError::Profile(format!("Failed to write config file: {e}")))?;

        Self::set_secure_permissions(&config_path)?;

        Ok(())
    }

    pub fn export_config(config: &Config, file_path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| OidcError::Profile(format!("Failed to serialize config: {e}")))?;

        fs::write(file_path, json)
            .map_err(|e| OidcError::Profile(format!("Failed to write export file: {e}")))?;

        Self::set_secure_permissions(file_path)?;

        Ok(())
    }

    pub fn import_config(file_path: &Path) -> Result<Config> {
        if !file_path.exists() {
            return Err(OidcError::Profile(format!(
                "Import file not found: {file_path:?}"
            )));
        }

        let content = fs::read_to_string(file_path)
            .map_err(|e| OidcError::Profile(format!("Failed to read import file: {e}")))?;

        let config: Config = serde_json::from_str(&content)
            .map_err(|e| OidcError::Profile(format!("Failed to parse import file: {e}")))?;

        for (name, profile) in &config.profiles {
            profile.validate().map_err(|e| {
                OidcError::Profile(format!("Invalid imported profile '{name}': {e}"))
            })?;
        }

        Ok(config)
    }

    #[cfg(unix)]
    fn set_secure_permissions(file_path: &Path) -> Result<()> {
        let metadata = fs::metadata(file_path)
            .map_err(|e| OidcError::Profile(format!("Failed to get file metadata: {e}")))?;

        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);

        fs::set_permissions(file_path, permissions)
            .map_err(|e| OidcError::Profile(format!("Failed to set file permissions: {e}")))?;

        Ok(())
    }

    #[cfg(windows)]
    fn set_secure_permissions(_file_path: &Path) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Profile;
    use tempfile::tempdir;

    fn create_test_config() -> Config {
        let mut config = Config::new();
        let profile = Profile {
            discovery_uri: Some("https://example.com/.well-known/openid-configuration".to_string()),
            client_id: "test-client".to_string(),
            client_secret: Some("test-secret".to_string()),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scope: "openid profile email".to_string(),
            authorization_endpoint: None,
            token_endpoint: None,
        };
        config.profiles.insert("test".to_string(), profile);
        config
    }

    #[test]
    fn test_export_import_config() {
        let temp_dir = tempdir().unwrap();
        let export_path = temp_dir.path().join("test_export.json");

        let original_config = create_test_config();

        ProfileStorage::export_config(&original_config, &export_path).unwrap();
        assert!(export_path.exists());

        let imported_config = ProfileStorage::import_config(&export_path).unwrap();

        assert_eq!(
            original_config.profiles.len(),
            imported_config.profiles.len()
        );
        assert!(imported_config.profiles.contains_key("test"));
    }

    #[test]
    fn test_import_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent.json");

        let result = ProfileStorage::import_config(&nonexistent_path);
        assert!(result.is_err());
    }
}
