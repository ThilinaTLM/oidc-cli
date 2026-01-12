use crate::auth::{OAuthClient, TokenExport};
use crate::browser::{BrowserOpener, WebBrowserOpener};
use crate::error::{OidcError, Result};
use crate::profile::ProfileManager;
use crate::server::CallbackServer;
use crate::ui::{display_tokens, handle_manual_code_entry, select_profile};
use crate::utils::url::{extract_port_from_redirect_uri, is_localhost_redirect_uri};
use std::path::PathBuf;
use tokio::time::{timeout, Duration};

/// Options for the login command
pub struct LoginOptions {
    pub profile_name: Option<String>,
    pub port: Option<u16>,
    pub copy: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub json: bool,
    pub output: Option<PathBuf>,
}

pub async fn handle_login(profile_manager: ProfileManager, options: LoginOptions) -> Result<()> {
    handle_login_with_browser_opener(profile_manager, options, &WebBrowserOpener).await
}

pub async fn handle_login_with_browser_opener<B: BrowserOpener>(
    profile_manager: ProfileManager,
    options: LoginOptions,
    browser_opener: &B,
) -> Result<()> {
    let LoginOptions {
        profile_name,
        port,
        copy,
        quiet,
        verbose,
        json,
        output,
    } = options;

    // --output implies --json
    let json_output = json || output.is_some();
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

    browser_opener.open_with_fallback(&auth_request.authorization_url, quiet)?;

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
        let output_clone = output.clone();

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
                    // Handle JSON output
                    if json_output {
                        output_tokens_json(&token_response, output_clone.as_ref(), quiet);
                    } else if quiet {
                        println!("{}", serde_json::to_string(&token_response).unwrap());
                    } else {
                        display_tokens(&token_response, copy).unwrap_or_else(|e| {
                            eprintln!("Error displaying tokens: {e}");
                        });
                    }

                    server_clone.set_tokens(token_response.clone()).await;

                    if !quiet && !json_output {
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

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    } else {
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

        // Handle JSON output
        if json_output {
            output_tokens_json(&token_response, output.as_ref(), quiet);
        } else if quiet {
            println!("{}", serde_json::to_string(&token_response).unwrap());
        } else {
            display_tokens(&token_response, copy)?;
        }
    }

    Ok(())
}

/// Output tokens as JSON to stdout or file
fn output_tokens_json(
    token_response: &crate::auth::TokenResponse,
    output_path: Option<&PathBuf>,
    quiet: bool,
) {
    let export = TokenExport::from_response(token_response);
    let json_str = serde_json::to_string_pretty(&export).unwrap();

    if let Some(path) = output_path {
        match std::fs::write(path, &json_str) {
            Ok(_) => {
                if !quiet {
                    println!("Tokens written to {}", path.display());
                }
            }
            Err(e) => {
                eprintln!("Error writing tokens to file: {e}");
            }
        }
    } else {
        println!("{json_str}");
    }
}
