use crate::error::{OidcError, Result};
use crate::profile::{ProfileManager, ProfileParams};
use crate::ui::prompts::*;
use std::io::{self, Write};

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

pub async fn handle_create(
    profile_manager: &mut ProfileManager,
    params: CreateParams,
) -> Result<()> {
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

pub async fn handle_edit(
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

pub fn handle_delete(
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

pub fn handle_rename(
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
