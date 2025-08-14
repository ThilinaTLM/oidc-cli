pub mod auth;
pub mod browser;
pub mod cli;
pub mod commands;
pub mod config;
pub mod crypto;
pub mod error;
pub mod profile;
pub mod server;
pub mod ui;
pub mod utils;

// Re-export main utilities for backward compatibility and testing
pub use utils::url::{extract_port_from_redirect_uri, is_localhost_redirect_uri, parse_query_params};

// Re-export from server.rs for testing
pub use server::parse_query_params as server_parse_query_params;
pub use server::{extract_path_from_redirect_uri, CallbackResult, CallbackServer};

// Re-export profile and browser modules for testing
pub use profile::ProfileManager;
#[cfg(test)]
pub use browser::MockBrowserOpener;