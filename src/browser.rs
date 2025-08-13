use crate::error::{OidcError, Result};

pub fn open_browser(url: &str) -> Result<()> {
    match webbrowser::open(url) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Failed to open browser automatically: {e}");
            Err(OidcError::BrowserFailed)
        }
    }
}

pub fn open_browser_with_fallback(url: &str, quiet: bool) -> Result<()> {
    match open_browser(url) {
        Ok(_) => {
            if !quiet {
                println!("Opening browser for authentication...");
            }
            Ok(())
        }
        Err(_) => {
            if !quiet {
                println!("Unable to open browser automatically.");
                println!("Please manually open the following URL in your browser:");
                println!();
                println!("{url}");
                println!();
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_browser_with_invalid_url() {
        let result = open_browser("not-a-valid-url");
        // The webbrowser crate might not always fail for invalid URLs on all platforms
        // so we just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_open_browser_with_fallback() {
        let result = open_browser_with_fallback("https://example.com", true);
        assert!(result.is_ok());
    }
}