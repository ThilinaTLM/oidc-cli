use std::collections::HashMap;
use url::Url;

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
