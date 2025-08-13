use oidc_cli::{extract_port_from_redirect_uri, is_localhost_redirect_uri, parse_query_params};

#[test]
fn test_is_localhost_redirect_uri() {
    assert!(is_localhost_redirect_uri("http://localhost:8080/callback"));
    assert!(is_localhost_redirect_uri("http://127.0.0.1:8383/callback"));
    assert!(is_localhost_redirect_uri("https://localhost/callback"));
    assert!(is_localhost_redirect_uri("http://[::1]:8080/callback"));
    assert!(!is_localhost_redirect_uri("https://example.com/callback"));
    assert!(!is_localhost_redirect_uri(
        "https://auth.company.com/callback"
    ));
    assert!(!is_localhost_redirect_uri("invalid-uri"));
}

#[test]
fn test_extract_port_from_redirect_uri() {
    assert_eq!(
        extract_port_from_redirect_uri("http://localhost:8383/callback"),
        Some(8383)
    );
    assert_eq!(
        extract_port_from_redirect_uri("http://127.0.0.1:9000/callback"),
        Some(9000)
    );
    assert_eq!(
        extract_port_from_redirect_uri("http://localhost/callback"),
        Some(80)
    );
    assert_eq!(
        extract_port_from_redirect_uri("https://localhost/callback"),
        Some(80)
    );
    assert_eq!(
        extract_port_from_redirect_uri("https://example.com/callback"),
        None
    );
    assert_eq!(extract_port_from_redirect_uri("invalid-uri"), None);
}

#[test]
fn test_parse_query_params() {
    let params = parse_query_params("code=abc123&state=xyz789&scope=openid%20profile");
    assert_eq!(params.get("code"), Some(&"abc123".to_string()));
    assert_eq!(params.get("state"), Some(&"xyz789".to_string()));
    assert_eq!(params.get("scope"), Some(&"openid profile".to_string()));

    let empty_params = parse_query_params("");
    assert!(empty_params.is_empty());
}
