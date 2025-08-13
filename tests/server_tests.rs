use oidc_cli::{extract_path_from_redirect_uri, server_parse_query_params, CallbackServer};

#[test]
fn test_parse_query_params() {
    let query = "code=abc123&state=xyz789&scope=openid%20profile";
    let params = server_parse_query_params(query);

    assert_eq!(params.get("code"), Some(&"abc123".to_string()));
    assert_eq!(params.get("state"), Some(&"xyz789".to_string()));
    assert_eq!(params.get("scope"), Some(&"openid profile".to_string()));
}

#[test]
fn test_callback_server_creation() {
    let server = CallbackServer::new(8080, "http://localhost:8080/callback");
    assert!(server.is_ok());

    let server = server.unwrap();
    assert_eq!(server.get_port(), 8080);
    assert_eq!(server.get_redirect_uri(), "http://127.0.0.1:8080/callback");
}

#[test]
fn test_extract_path_from_redirect_uri() {
    assert_eq!(
        extract_path_from_redirect_uri("http://localhost:8383/docs/ui"),
        "/docs/ui"
    );
    assert_eq!(
        extract_path_from_redirect_uri("http://localhost:8080/callback"),
        "/callback"
    );
    assert_eq!(
        extract_path_from_redirect_uri("http://localhost:8080/"),
        "/"
    );
    assert_eq!(extract_path_from_redirect_uri("invalid-uri"), "/callback");
}

#[tokio::test]
async fn test_callback_server_start() {
    let mut server = CallbackServer::new(0, "http://localhost:8080/callback").unwrap(); // Use port 0 for automatic assignment
    let receiver = server.start().await;
    assert!(receiver.is_ok());
}
