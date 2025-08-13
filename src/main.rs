mod auth;
mod browser;
mod cli;
mod config;
mod crypto;
mod error;
mod profile;
mod server;

use clap::Parser;
use cli::{Cli, Commands};
use error::{OidcError, Result};
use profile::{ProfileManager, ProfileParams};
use std::collections::HashMap;
use std::io::{self, Write};
use tokio::time::{timeout, Duration};
use url::Url;

use auth::OAuthClient;
use browser::open_browser_with_fallback;
use server::CallbackServer;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        if !matches!(e, OidcError::Cancelled) {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    let mut profile_manager = ProfileManager::new()?;

    let is_quiet = cli.is_quiet();
    let is_verbose = cli.is_verbose();

    match cli.command {
        Commands::Login {
            profile,
            port,
            copy,
        } => handle_login(profile_manager, profile, port, copy, is_quiet, is_verbose).await,
        Commands::List => handle_list(profile_manager, is_quiet),
        Commands::Create {
            name,
            client_id,
            client_secret,
            redirect_uri,
            scope,
            discovery_uri,
            auth_endpoint,
            token_endpoint,
            non_interactive,
        } => {
            handle_create(
                &mut profile_manager,
                CreateParams {
                    name,
                    client_id,
                    client_secret,
                    redirect_uri,
                    scope,
                    discovery_uri,
                    auth_endpoint,
                    token_endpoint,
                    non_interactive,
                    quiet: is_quiet,
                },
            )
            .await
        }
        Commands::Edit { name } => handle_edit(&mut profile_manager, name, is_quiet).await,
        Commands::Delete { name, force } => {
            handle_delete(&mut profile_manager, name, force, is_quiet)
        }
        Commands::Rename { old_name, new_name } => {
            handle_rename(&mut profile_manager, old_name, new_name, is_quiet)
        }
        Commands::Export { file, profiles } => {
            handle_export(profile_manager, file, profiles, is_quiet)
        }
        Commands::Import { file, overwrite } => {
            handle_import(&mut profile_manager, file, overwrite, is_quiet)
        }
    }
}

async fn handle_login(
    profile_manager: ProfileManager,
    profile_name: Option<String>,
    port: Option<u16>,
    copy: bool,
    quiet: bool,
    verbose: bool,
) -> Result<()> {
    let profile_name = match profile_name {
        Some(name) => name,
        None => select_profile(&profile_manager, quiet)?,
    };

    let profile = profile_manager.get_profile(&profile_name)?.clone();

    let oauth_client = OAuthClient::new(profile.clone()).await?;
    let auth_request = oauth_client.create_authorization_request()?;

    if !quiet {
        println!("Initiating OAuth 2.0 authorization flow...");
    }

    open_browser_with_fallback(&auth_request.authorization_url, quiet)?;

    let (code, state, server_opt) = if is_localhost_redirect_uri(&profile.redirect_uri) {
        // Use callback server for localhost URLs
        let port = port
            .or_else(|| extract_port_from_redirect_uri(&profile.redirect_uri))
            .unwrap_or(8080);

        let mut server = CallbackServer::new(port, &profile.redirect_uri)?;

        if verbose {
            println!("Starting callback server on port {port}");
        }

        let mut receiver = server.start().await?;

        if !quiet {
            println!("Waiting for authentication callback...");
            println!("Press Ctrl+C to cancel");
        }

        let callback_result = timeout(Duration::from_secs(300), receiver.recv())
            .await
            .map_err(|_| OidcError::Auth("Authentication timeout (5 minutes)".to_string()))?
            .ok_or_else(|| OidcError::Auth("Failed to receive callback".to_string()))?;

        if let Some(error) = callback_result.error {
            return Err(OidcError::Auth(format!(
                "Authentication failed: {} - {}",
                error,
                callback_result.error_description.unwrap_or_default()
            )));
        }

        (callback_result.code, callback_result.state, Some(server))
    } else {
        let code = handle_manual_code_entry(quiet).await?;
        (code, auth_request.state.clone(), None)
    };

    if let Some(server) = server_opt {
        // Exchange tokens in background while browser shows success page
        let server_clone = server.clone();
        let oauth_client_clone = oauth_client.clone();
        let code_clone = code.clone();
        let state_clone = state.clone();
        let auth_state_clone = auth_request.state.clone();
        let verifier_clone = auth_request.pkce_challenge.verifier.clone();

        tokio::spawn(async move {
            if verbose {
                println!("Received authorization code, exchanging for tokens...");
            }

            match oauth_client_clone
                .exchange_code_for_tokens(
                    &code_clone,
                    &state_clone,
                    &auth_state_clone,
                    &verifier_clone,
                )
                .await
            {
                Ok(token_response) => {
                    // Display tokens in terminal
                    if quiet {
                        println!("{}", serde_json::to_string(&token_response).unwrap());
                    } else {
                        display_tokens(&token_response, copy).unwrap_or_else(|e| {
                            eprintln!("Error displaying tokens: {e}");
                        });
                    }

                    // Set token on server so browser can access it
                    server_clone
                        .set_token(token_response.access_token.clone())
                        .await;

                    if !quiet {
                        println!();
                        println!("Token is now available in the browser.");
                    }
                }
                Err(e) => {
                    eprintln!("Error exchanging code for tokens: {e}");
                }
            }
        });

        if !quiet {
            println!("Authentication successful! Check your browser for the access token.");
            println!("Tokens will be displayed in the browser once ready...");
        }

        // Wait a shorter time for token to be available, then exit
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    } else {
        // For non-localhost redirect URIs, exchange tokens normally
        if verbose {
            println!("Received authorization code, exchanging for tokens...");
        }

        let token_response = oauth_client
            .exchange_code_for_tokens(
                &code,
                &state,
                &auth_request.state,
                &auth_request.pkce_challenge.verifier,
            )
            .await?;

        // Display tokens immediately after successful exchange
        if quiet {
            println!("{}", serde_json::to_string(&token_response).unwrap());
        } else {
            display_tokens(&token_response, copy)?;
        }
    }

    Ok(())
}

fn select_profile(profile_manager: &ProfileManager, quiet: bool) -> Result<String> {
    let profiles = profile_manager.list_profiles();

    if profiles.is_empty() {
        return Err(OidcError::Profile(
            "No profiles found. Create a profile first using 'create' command.".to_string(),
        ));
    }

    if profiles.len() == 1 {
        return Ok(profiles[0].clone());
    }

    if quiet {
        return Err(OidcError::Profile(
            "Multiple profiles available. Please specify a profile name.".to_string(),
        ));
    }

    println!("Multiple profiles available:");
    for (i, profile) in profiles.iter().enumerate() {
        println!("  {}. {}", i + 1, profile);
    }

    loop {
        print!("Select a profile (1-{}): ", profiles.len());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice > 0 && choice <= profiles.len() {
                return Ok(profiles[choice - 1].clone());
            }
        }

        println!(
            "Invalid selection. Please enter a number between 1 and {}.",
            profiles.len()
        );
    }
}

fn display_tokens(token_response: &auth::TokenResponse, copy: bool) -> Result<()> {
    println!("🎉 Authentication successful!");
    println!();
    println!("Access Token: {}", token_response.access_token);

    if let Some(ref token_type) = Some(&token_response.token_type) {
        println!("Token Type: {token_type}");
    }

    if let Some(expires_in) = token_response.expires_in {
        println!("Expires In: {expires_in} seconds");
    }

    println!();

    if let Some(ref refresh_token) = token_response.refresh_token {
        println!("Refresh Token: {refresh_token}");
    }

    if let Some(ref id_token) = token_response.id_token {
        println!("ID Token: {id_token}");
    }

    if let Some(ref scope) = token_response.scope {
        println!("Scope: {scope}");
    }

    if copy {
        #[cfg(feature = "clipboard")]
        {
            use clipboard::{ClipboardContext, ClipboardProvider};
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            ctx.set_contents(token_response.access_token.clone())
                .unwrap();
            println!();
            println!("Access token copied to clipboard!");
        }
        #[cfg(not(feature = "clipboard"))]
        {
            println!();
            println!("Clipboard feature not available in this build.");
        }
    }

    Ok(())
}

fn handle_list(profile_manager: ProfileManager, quiet: bool) -> Result<()> {
    let profiles = profile_manager.list_profiles();

    if profiles.is_empty() {
        if !quiet {
            println!("No profiles found.");
        }
        return Ok(());
    }

    if quiet {
        for profile in profiles {
            println!("{profile}");
        }
    } else {
        println!("Available profiles:");
        for profile in profiles {
            println!("  • {profile}");
        }
    }

    Ok(())
}

pub struct CreateParams {
    pub name: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub discovery_uri: Option<String>,
    pub auth_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub non_interactive: bool,
    pub quiet: bool,
}

async fn handle_create(profile_manager: &mut ProfileManager, params: CreateParams) -> Result<()> {
    if params.non_interactive {
        let client_id = params.client_id.ok_or_else(|| {
            OidcError::Config("--client-id is required in non-interactive mode".to_string())
        })?;
        let redirect_uri = params.redirect_uri.ok_or_else(|| {
            OidcError::Config("--redirect-uri is required in non-interactive mode".to_string())
        })?;
        let scope = params.scope.ok_or_else(|| {
            OidcError::Config("--scope is required in non-interactive mode".to_string())
        })?;

        if params.discovery_uri.is_none()
            && (params.auth_endpoint.is_none() || params.token_endpoint.is_none())
        {
            return Err(OidcError::Config("Either --discovery-uri or both --auth-endpoint and --token-endpoint are required in non-interactive mode".to_string()));
        }

        profile_manager.create_profile(ProfileParams {
            name: params.name.clone(),
            client_id,
            client_secret: params.client_secret,
            redirect_uri,
            scope,
            discovery_uri: params.discovery_uri,
            authorization_endpoint: params.auth_endpoint,
            token_endpoint: params.token_endpoint,
        })?;

        if !params.quiet {
            println!("Profile '{}' created successfully.", params.name);
        }
    } else {
        create_profile_interactive(profile_manager, params.name, params.quiet).await?;
    }

    Ok(())
}

async fn create_profile_interactive(
    profile_manager: &mut ProfileManager,
    name: String,
    quiet: bool,
) -> Result<()> {
    if !quiet {
        println!("Creating new profile '{name}'");
        println!("Press Ctrl+C to cancel at any time");
        println!();
    }

    let client_id = prompt_input("Client ID", true)?;
    let client_secret = prompt_optional_input("Client Secret (optional)")?;
    let redirect_uri = prompt_input_with_default("Redirect URI", "http://localhost:8080/callback")?;
    let scope = prompt_input_with_default("Scope", "openid profile email")?;

    println!();
    println!("Choose configuration method:");
    println!("  1. Use discovery URI (recommended)");
    println!("  2. Manual endpoint configuration");

    let use_discovery = loop {
        print!("Select option (1-2): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => break true,
            "2" => break false,
            _ => println!("Invalid selection. Please enter 1 or 2."),
        }
    };

    let (discovery_uri, auth_endpoint, token_endpoint) = if use_discovery {
        let discovery_uri = prompt_input("Discovery URI", true)?;
        (Some(discovery_uri), None, None)
    } else {
        let auth_endpoint = prompt_input("Authorization Endpoint", true)?;
        let token_endpoint = prompt_input("Token Endpoint", true)?;
        (None, Some(auth_endpoint), Some(token_endpoint))
    };

    profile_manager.create_profile(ProfileParams {
        name: name.clone(),
        client_id,
        client_secret,
        redirect_uri,
        scope,
        discovery_uri,
        authorization_endpoint: auth_endpoint,
        token_endpoint,
    })?;

    if !quiet {
        println!();
        println!("✓ Profile '{name}' created successfully!");
    }

    Ok(())
}

async fn handle_edit(
    profile_manager: &mut ProfileManager,
    name: String,
    quiet: bool,
) -> Result<()> {
    let profile = profile_manager.get_profile(&name)?.clone();

    if !quiet {
        println!("Editing profile '{name}'");
        println!("Press Enter to keep current value, or enter new value:");
        println!();
    }

    let client_id = prompt_input_with_current("Client ID", &profile.client_id)?;
    let client_secret = if profile.client_secret.is_some() {
        prompt_optional_input_with_current("Client Secret", profile.client_secret.as_deref())?
    } else {
        prompt_optional_input("Client Secret (optional)")?
    };
    let redirect_uri = prompt_input_with_current("Redirect URI", &profile.redirect_uri)?;
    let scope = prompt_input_with_current("Scope", &profile.scope)?;

    let (discovery_uri, auth_endpoint, token_endpoint) = if profile.discovery_uri.is_some() {
        let discovery_uri =
            prompt_optional_input_with_current("Discovery URI", profile.discovery_uri.as_deref())?;
        (discovery_uri, None, None)
    } else {
        let auth_endpoint = prompt_optional_input_with_current(
            "Authorization Endpoint",
            profile.authorization_endpoint.as_deref(),
        )?;
        let token_endpoint = prompt_optional_input_with_current(
            "Token Endpoint",
            profile.token_endpoint.as_deref(),
        )?;
        (None, auth_endpoint, token_endpoint)
    };

    profile_manager.update_profile(ProfileParams {
        name: name.clone(),
        client_id,
        client_secret,
        redirect_uri,
        scope,
        discovery_uri,
        authorization_endpoint: auth_endpoint,
        token_endpoint,
    })?;

    if !quiet {
        println!("✓ Profile '{name}' updated successfully!");
    }

    Ok(())
}

fn handle_delete(
    profile_manager: &mut ProfileManager,
    name: String,
    force: bool,
    quiet: bool,
) -> Result<()> {
    profile_manager.get_profile(&name)?;

    if !force && !quiet {
        print!("Are you sure you want to delete profile '{name}'? (y/N): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

    profile_manager.delete_profile(&name)?;

    if !quiet {
        println!("✓ Profile '{name}' deleted successfully.");
    }

    Ok(())
}

fn handle_rename(
    profile_manager: &mut ProfileManager,
    old_name: String,
    new_name: String,
    quiet: bool,
) -> Result<()> {
    profile_manager.rename_profile(&old_name, new_name.clone())?;

    if !quiet {
        println!("✓ Profile '{old_name}' renamed to '{new_name}' successfully.");
    }

    Ok(())
}

fn handle_export(
    profile_manager: ProfileManager,
    file: std::path::PathBuf,
    profiles: Vec<String>,
    quiet: bool,
) -> Result<()> {
    let profile_names = if profiles.is_empty() {
        None
    } else {
        for name in &profiles {
            profile_manager.get_profile(name)?;
        }
        Some(profiles)
    };

    profile_manager.export_profiles(&file, profile_names)?;

    if !quiet {
        println!("✓ Profiles exported to {file:?} successfully.");
    }

    Ok(())
}

fn handle_import(
    profile_manager: &mut ProfileManager,
    file: std::path::PathBuf,
    overwrite: bool,
    quiet: bool,
) -> Result<()> {
    if !file.exists() {
        return Err(OidcError::Profile(format!(
            "Import file not found: {file:?}"
        )));
    }

    let imported_names = profile_manager.import_profiles(&file, overwrite)?;

    if !quiet {
        println!(
            "✓ Imported {} profile(s) from {:?}:",
            imported_names.len(),
            file
        );
        for name in imported_names {
            println!("  • {name}");
        }
    }

    Ok(())
}

pub fn is_localhost_redirect_uri(uri: &str) -> bool {
    if let Ok(url) = Url::parse(uri) {
        if let Some(host) = url.host() {
            match host {
                url::Host::Domain(domain) => {
                    return domain == "localhost";
                }
                url::Host::Ipv4(addr) => {
                    return addr.is_loopback();
                }
                url::Host::Ipv6(addr) => {
                    return addr.is_loopback();
                }
            }
        }
    }
    false
}

pub fn extract_port_from_redirect_uri(uri: &str) -> Option<u16> {
    if let Ok(url) = Url::parse(uri) {
        if is_localhost_redirect_uri(uri) {
            return url.port().or(Some(80));
        }
    }
    None
}

pub fn parse_query_params(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if let (Ok(decoded_key), Ok(decoded_value)) =
                (urlencoding::decode(key), urlencoding::decode(value))
            {
                params.insert(decoded_key.to_string(), decoded_value.to_string());
            }
        }
    }

    params
}

async fn handle_manual_code_entry(quiet: bool) -> Result<String> {
    if !quiet {
        println!("Since your redirect URI is not localhost, you'll need to manually enter the authorization code.");
        println!("After authorizing in your browser, copy the full callback URL or just the 'code' parameter.");
    }

    loop {
        print!("Enter the authorization code or full callback URL: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            println!("Authorization code cannot be empty. Please try again.");
            continue;
        }

        // Try to parse as a URL first
        if let Ok(url) = Url::parse(input) {
            if let Some(query) = url.query() {
                let params = parse_query_params(query);
                if let Some(code) = params.get("code") {
                    return Ok(code.clone());
                }
            }
        }

        // If not a URL, treat as direct code
        if !input.contains("://") {
            return Ok(input.to_string());
        }

        println!("Could not extract authorization code from the input. Please try again.");
    }
}

fn prompt_input(prompt: &str, required: bool) -> Result<String> {
    loop {
        print!("{prompt}: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() && required {
            println!("This field is required. Please enter a value.");
            continue;
        }

        return Ok(input.to_string());
    }
}

fn prompt_input_with_default(prompt: &str, default: &str) -> Result<String> {
    print!("{prompt} [{default}]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}

fn prompt_input_with_current(prompt: &str, current: &str) -> Result<String> {
    print!("{prompt} [{current}]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(current.to_string())
    } else {
        Ok(input.to_string())
    }
}

fn prompt_optional_input(prompt: &str) -> Result<Option<String>> {
    print!("{prompt}: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input.to_string()))
    }
}

fn prompt_optional_input_with_current(
    prompt: &str,
    current: Option<&str>,
) -> Result<Option<String>> {
    let display_current = current.unwrap_or("none");
    print!("{prompt} [{display_current}]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(current.map(|s| s.to_string()))
    } else if input == "none" || input == "null" {
        Ok(None)
    } else {
        Ok(Some(input.to_string()))
    }
}
