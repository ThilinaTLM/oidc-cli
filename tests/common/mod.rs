// Common test utilities for the OIDC CLI project

use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub config_dir: PathBuf,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let config_dir = temp_dir.path().to_path_buf();
        
        Self {
            temp_dir,
            config_dir,
        }
    }
    
    pub fn config_dir_path(&self) -> PathBuf {
        self.config_dir.clone()
    }
}

#[cfg(test)]
pub fn create_test_profile_manager_with_temp_dir() -> (oidc_cli::profile::ProfileManager, TestEnvironment) {
    let test_env = TestEnvironment::new();
    let manager = oidc_cli::profile::ProfileManager::new_with_test_dir(Some(test_env.config_dir_path()))
        .expect("Failed to create test profile manager");
    (manager, test_env)
}

#[cfg(test)]  
pub fn create_mock_browser_opener() -> oidc_cli::browser::MockBrowserOpener {
    oidc_cli::browser::MockBrowserOpener::new()
}

/// Helper function to create test query parameters
pub fn create_test_query_params() -> String {
    "code=test_code&state=test_state&scope=openid%20profile".to_string()
}

/// Helper function to create expected parsed parameters
pub fn create_expected_params() -> HashMap<String, String> {
    let mut params = HashMap::new();
    params.insert("code".to_string(), "test_code".to_string());
    params.insert("state".to_string(), "test_state".to_string());
    params.insert("scope".to_string(), "openid profile".to_string());
    params
}

/// Helper function to create test redirect URIs
pub fn get_test_redirect_uris() -> Vec<(&'static str, bool)> {
    vec![
        ("http://localhost:8080/callback", true),
        ("http://127.0.0.1:8080/callback", true),
        ("https://localhost/callback", true),
        ("http://[::1]:8080/callback", true),
        ("https://example.com/callback", false),
        ("https://auth.company.com/callback", false),
        ("invalid-uri", false),
    ]
}