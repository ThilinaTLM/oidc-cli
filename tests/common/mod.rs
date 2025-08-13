// Common test utilities for the OIDC CLI project

use std::collections::HashMap;

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