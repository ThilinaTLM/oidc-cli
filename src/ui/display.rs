use crate::auth;
use crate::error::Result;

pub fn display_tokens(token_response: &auth::TokenResponse, copy: bool) -> Result<()> {
    println!("🎉 Authentication successful!");
    println!();

    println!("=== TOKENS ===");
    println!();

    println!("Access Token:");
    println!("{}", token_response.access_token);
    println!("Type: {}", token_response.token_type);
    if let Some(expires_in) = token_response.expires_in {
        println!("Expires In: {expires_in} seconds");
    } else {
        println!("Expires In: Not specified");
    }
    println!();

    if let Some(ref id_token) = token_response.id_token {
        println!("ID Token:");
        println!("{id_token}");
        if let Some(expires_in) = token_response.expires_in {
            println!("Expires In: {expires_in} seconds (same as access token)");
        } else {
            println!("Expires In: Check token 'exp' claim for exact expiration");
        }
        println!();
    }

    if let Some(ref refresh_token) = token_response.refresh_token {
        println!("Refresh Token:");
        println!("{refresh_token}");
        println!();
    }

    if let Some(ref scope) = token_response.scope {
        println!("Scope: {scope}");
        println!();
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
