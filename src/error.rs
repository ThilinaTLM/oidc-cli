#![allow(dead_code)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum OidcError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Profile error: {0}")]
    Profile(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("State parameter mismatch")]
    StateMismatch,

    #[error("Invalid redirect URI: {0}")]
    InvalidRedirectUri(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid token response")]
    InvalidTokenResponse,

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Profile already exists: {0}")]
    ProfileExists(String),

    #[error("Discovery failed: {0}")]
    Discovery(String),

    #[error("Browser opening failed")]
    BrowserFailed,

    #[error("Operation cancelled by user")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, OidcError>;
