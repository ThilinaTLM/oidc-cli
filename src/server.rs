use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use url::Url;

use crate::error::Result;

fn extract_path_from_redirect_uri(redirect_uri: &str) -> String {
    if let Ok(url) = Url::parse(redirect_uri) {
        url.path().to_string()
    } else {
        "/callback".to_string() // fallback to default
    }
}

pub struct CallbackResult {
    pub code: String,
    pub state: String,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

pub struct CallbackServer {
    addr: SocketAddr,
    sender: Option<mpsc::Sender<CallbackResult>>,
    callback_path: String,
}

impl CallbackServer {
    pub fn new(port: u16, redirect_uri: &str) -> Result<Self> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let callback_path = extract_path_from_redirect_uri(redirect_uri);
        Ok(CallbackServer {
            addr,
            sender: None,
            callback_path,
        })
    }

    pub async fn start(&mut self) -> Result<mpsc::Receiver<CallbackResult>> {
        let (tx, rx) = mpsc::channel::<CallbackResult>(1);
        self.sender = Some(tx.clone());

        let tx_arc = Arc::new(tx);
        let addr = self.addr;
        let callback_path = Arc::new(self.callback_path.clone());
        
        let make_svc = make_service_fn(move |_conn| {
            let tx = tx_arc.clone();
            let path = callback_path.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    handle_request(req, tx.clone(), path.clone())
                }))
            }
        });

        tokio::spawn(async move {
            let server = Server::bind(&addr).serve(make_svc);
            
            if let Err(e) = server.await {
                eprintln!("Server error: {e}");
            }
        });

        Ok(rx)
    }

    #[allow(dead_code)]
    pub fn get_redirect_uri(&self) -> String {
        format!("http://{}:{}{}", self.addr.ip(), self.addr.port(), self.callback_path)
    }

    #[allow(dead_code)]
    pub fn get_port(&self) -> u16 {
        self.addr.port()
    }
}

async fn handle_request(
    req: Request<Body>,
    tx: Arc<mpsc::Sender<CallbackResult>>,
    callback_path: Arc<String>,
) -> std::result::Result<Response<Body>, Infallible> {
    match req.method() {
        &Method::GET => {
            let uri = req.uri();
            
            if uri.path() == callback_path.as_str() {
                if let Some(query) = uri.query() {
                    let params = parse_query_params(query);
                    
                    if let Some(error) = params.get("error") {
                        let error_description = params.get("error_description").cloned();
                        let error_desc_ref = error_description.as_deref();
                        let result = CallbackResult {
                            code: String::new(),
                            state: params.get("state").cloned().unwrap_or_default(),
                            error: Some(error.clone()),
                            error_description: error_description.clone(),
                        };
                        
                        let _ = tx.send(result).await;
                        return Ok(create_error_response(error, error_desc_ref));
                    }
                    
                    if let (Some(code), Some(state)) = (params.get("code"), params.get("state")) {
                        let result = CallbackResult {
                            code: code.clone(),
                            state: state.clone(),
                            error: None,
                            error_description: None,
                        };
                        
                        let _ = tx.send(result).await;
                        return Ok(create_success_response());
                    }
                }
                
                return Ok(create_error_response_with_status(
                    StatusCode::BAD_REQUEST,
                    "Missing required parameters"
                ));
            }
            
            Ok(create_error_response_with_status(
                StatusCode::NOT_FOUND,
                "Not Found"
            ))
        }
        _ => Ok(create_error_response_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            "Method Not Allowed"
        )),
    }
}

fn parse_query_params(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    
    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if let (Ok(decoded_key), Ok(decoded_value)) = (
                urlencoding::decode(key),
                urlencoding::decode(value)
            ) {
                params.insert(decoded_key.to_string(), decoded_value.to_string());
            }
        }
    }
    
    params
}

fn create_success_response() -> Response<Body> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Authentication Successful</title>
    <style>
        body { 
            font-family: Arial, sans-serif; 
            display: flex; 
            justify-content: center; 
            align-items: center; 
            height: 100vh; 
            margin: 0; 
            background-color: #f5f5f5; 
        }
        .container { 
            text-align: center; 
            background: white; 
            padding: 2rem; 
            border-radius: 8px; 
            box-shadow: 0 2px 10px rgba(0,0,0,0.1); 
        }
        .success { color: #28a745; }
        .message { margin-top: 1rem; color: #666; }
    </style>
</head>
<body>
    <div class="container">
        <h1 class="success">✓ Authentication Successful!</h1>
        <p class="message">You can now close this browser window and return to the terminal.</p>
    </div>
</body>
</html>
"#;

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .body(Body::from(html))
        .unwrap()
}

fn create_error_response(error: &str, error_description: Option<&str>) -> Response<Body> {
    let description = error_description.unwrap_or("An authentication error occurred");
    
    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Authentication Error</title>
    <style>
        body {{ 
            font-family: Arial, sans-serif; 
            display: flex; 
            justify-content: center; 
            align-items: center; 
            height: 100vh; 
            margin: 0; 
            background-color: #f5f5f5; 
        }}
        .container {{ 
            text-align: center; 
            background: white; 
            padding: 2rem; 
            border-radius: 8px; 
            box-shadow: 0 2px 10px rgba(0,0,0,0.1); 
        }}
        .error {{ color: #dc3545; }}
        .message {{ margin-top: 1rem; color: #666; }}
        .details {{ margin-top: 1rem; padding: 1rem; background: #f8f9fa; border-radius: 4px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1 class="error">✗ Authentication Failed</h1>
        <p class="message">Please close this browser window and try again.</p>
        <div class="details">
            <strong>Error:</strong> {error}<br>
            <strong>Description:</strong> {description}
        </div>
    </div>
</body>
</html>
"#);

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .body(Body::from(html))
        .unwrap()
}

fn create_error_response_with_status(status: StatusCode, message: &str) -> Response<Body> {
    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Error</title>
    <style>
        body {{ 
            font-family: Arial, sans-serif; 
            display: flex; 
            justify-content: center; 
            align-items: center; 
            height: 100vh; 
            margin: 0; 
            background-color: #f5f5f5; 
        }}
        .container {{ 
            text-align: center; 
            background: white; 
            padding: 2rem; 
            border-radius: 8px; 
            box-shadow: 0 2px 10px rgba(0,0,0,0.1); 
        }}
        .error {{ color: #dc3545; }}
    </style>
</head>
<body>
    <div class="container">
        <h1 class="error">{} - {}</h1>
    </div>
</body>
</html>
"#, status.as_u16(), message);

    Response::builder()
        .status(status)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_params() {
        let query = "code=abc123&state=xyz789&scope=openid%20profile";
        let params = parse_query_params(query);
        
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
        assert_eq!(extract_path_from_redirect_uri("http://localhost:8383/docs/ui"), "/docs/ui");
        assert_eq!(extract_path_from_redirect_uri("http://localhost:8080/callback"), "/callback");
        assert_eq!(extract_path_from_redirect_uri("http://localhost:8080/"), "/");
        assert_eq!(extract_path_from_redirect_uri("invalid-uri"), "/callback");
    }

    #[tokio::test]
    async fn test_callback_server_start() {
        let mut server = CallbackServer::new(0, "http://localhost:8080/callback").unwrap(); // Use port 0 for automatic assignment
        let receiver = server.start().await;
        assert!(receiver.is_ok());
    }
}