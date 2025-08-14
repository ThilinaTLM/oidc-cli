use crate::error::{OidcError, Result};

pub trait BrowserOpener {
    fn open_with_fallback(&self, url: &str, quiet: bool) -> Result<()>;
}

pub struct WebBrowserOpener;

impl BrowserOpener for WebBrowserOpener {
    fn open_with_fallback(&self, url: &str, quiet: bool) -> Result<()> {
        open_browser_with_fallback(url, quiet)
    }
}

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
pub struct MockBrowserOpener {
    pub opened_urls: std::sync::Mutex<Vec<String>>,
}

#[cfg(test)]
impl MockBrowserOpener {
    pub fn new() -> Self {
        Self {
            opened_urls: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn get_opened_urls(&self) -> Vec<String> {
        self.opened_urls.lock().unwrap().clone()
    }
}

#[cfg(test)]
impl BrowserOpener for MockBrowserOpener {
    fn open_with_fallback(&self, url: &str, quiet: bool) -> Result<()> {
        self.opened_urls.lock().unwrap().push(url.to_string());
        if !quiet {
            println!("Mock: Would open browser for authentication...");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_browser_opener_with_invalid_url() {
        let mock = MockBrowserOpener::new();
        let result = mock.open_with_fallback("not-a-valid-url", true);
        // Mock implementation should always succeed
        assert!(result.is_ok());
        
        let urls = mock.get_opened_urls();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "not-a-valid-url");
    }

    #[test]
    fn test_mock_browser_opener_with_fallback() {
        let mock = MockBrowserOpener::new();
        let result = mock.open_with_fallback("https://example.com", true);
        assert!(result.is_ok());
        
        let urls = mock.get_opened_urls();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com");
    }

    #[test]
    fn test_mock_browser_opener() {
        let mock = MockBrowserOpener::new();
        
        assert!(mock.open_with_fallback("https://example.com", true).is_ok());
        assert!(mock.open_with_fallback("https://test.com", true).is_ok());
        
        let urls = mock.get_opened_urls();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://example.com");
        assert_eq!(urls[1], "https://test.com");
    }
}
