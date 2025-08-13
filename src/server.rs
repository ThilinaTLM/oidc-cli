use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use url::Url;

use crate::error::Result;

pub fn extract_path_from_redirect_uri(redirect_uri: &str) -> String {
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
    #[allow(dead_code)]
    pub access_token: Option<String>,
}

#[derive(Clone)]
pub struct CallbackServer {
    addr: SocketAddr,
    sender: Option<mpsc::Sender<CallbackResult>>,
    callback_path: String,
    token_store: Arc<RwLock<Option<String>>>,
}

impl CallbackServer {
    pub fn new(port: u16, redirect_uri: &str) -> Result<Self> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let callback_path = extract_path_from_redirect_uri(redirect_uri);
        Ok(CallbackServer {
            addr,
            sender: None,
            callback_path,
            token_store: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn start(&mut self) -> Result<mpsc::Receiver<CallbackResult>> {
        let (tx, rx) = mpsc::channel::<CallbackResult>(1);
        self.sender = Some(tx.clone());

        let tx_arc = Arc::new(tx);
        let addr = self.addr;
        let callback_path = Arc::new(self.callback_path.clone());
        let token_store = self.token_store.clone();

        let make_svc = make_service_fn(move |_conn| {
            let tx = tx_arc.clone();
            let path = callback_path.clone();
            let store = token_store.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    handle_request(req, tx.clone(), path.clone(), store.clone())
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
        format!(
            "http://{}:{}{}",
            self.addr.ip(),
            self.addr.port(),
            self.callback_path
        )
    }

    #[allow(dead_code)]
    pub fn get_port(&self) -> u16 {
        self.addr.port()
    }

    pub async fn set_token(&self, token: String) {
        let mut store = self.token_store.write().await;
        *store = Some(token);
    }
}

async fn handle_request(
    req: Request<Body>,
    tx: Arc<mpsc::Sender<CallbackResult>>,
    callback_path: Arc<String>,
    token_store: Arc<RwLock<Option<String>>>,
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
                            access_token: None,
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
                            access_token: None,
                        };

                        let _ = tx.send(result).await;

                        // Always serve success page immediately, let JavaScript polling handle token display
                        return Ok(create_success_response());
                    }
                }

                return Ok(create_error_response_with_status(
                    StatusCode::BAD_REQUEST,
                    "Missing required parameters",
                ));
            }

            // Handle token endpoint
            if uri.path() == "/token" {
                let token_guard = token_store.read().await;
                if let Some(ref token) = *token_guard {
                    return Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json; charset=utf-8")
                        .header("Cache-Control", "no-cache, no-store, must-revalidate")
                        .body(Body::from(format!(r#"{{"token":"{token}"}}"#)))
                        .unwrap());
                } else {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header("Content-Type", "application/json; charset=utf-8")
                        .body(Body::from(r#"{"error":"Token not available"}"#))
                        .unwrap());
                }
            }

            Ok(create_error_response_with_status(
                StatusCode::NOT_FOUND,
                "Not Found",
            ))
        }
        _ => Ok(create_error_response_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            "Method Not Allowed",
        )),
    }
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

fn create_success_response() -> Response<Body> {
    create_success_response_with_token(None)
}

fn create_success_response_with_token(access_token: Option<&str>) -> Response<Body> {
    let mut html = include_str!("templates/success.html").to_string();

    if let Some(token) = access_token {
        html = html.replace("{access_token}", token);
        html = html.replace("{show_copy_button}", "true");
    } else {
        html = html.replace("{access_token}", "");
        html = html.replace("{show_copy_button}", "false");
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .body(Body::from(html))
        .unwrap()
}

fn create_error_response(error: &str, error_description: Option<&str>) -> Response<Body> {
    let description = error_description.unwrap_or("An authentication error occurred");

    let html = include_str!("templates/error.html")
        .replace("{error}", error)
        .replace("{description}", description);

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .body(Body::from(html))
        .unwrap()
}

fn create_error_response_with_status(status: StatusCode, message: &str) -> Response<Body> {
    let html = include_str!("templates/generic_error.html")
        .replace("{status_code}", &status.as_u16().to_string())
        .replace("{message}", message);

    Response::builder()
        .status(status)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}
