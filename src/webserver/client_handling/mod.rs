//! Client Module
//!
//! This module defines the `Client` struct, which represents a single connection
//! to the web server. It handles reading HTTP and TLS requests, applying
//! middleware, routing to the correct handler, and sending responses.
//!
//! # Features
//! - HTTP/1.1 request parsing
//! - Optional TLS support via `rustls`
//! - Middleware support for request/response modification
//! - Static file serving, custom routes, and reverse proxying
//! - CORS and security headers application
//!
//! # Example
//! ```no_run
//! use std::net::TcpListener;
//! use std::sync::{Arc, Mutex};
//! use my_crate::webserver::{Client, Domain, Route};
//!
//! let domains = Arc::new(Mutex::new(HashMap::new()));
//! let default_domain = Domain::new("localhost");
//! let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
//!
//! for stream in listener.incoming() {
//!     if let Ok(stream) = stream {
//!         let mut client = Client::new(stream, domains.clone(), default_domain.clone(), Arc::new(Vec::new()), None);
//!         client.handle(0);
//!     }
//! }
//! ```
use crate::webserver::Domain;
use crate::webserver::files::get_static_file_content;
use crate::webserver::http_packet::header::connection::ConnectionType;
use crate::webserver::http_packet::header::content_types::ContentType;
use crate::webserver::middleware::{Middleware, MiddlewareFn};
use crate::webserver::proxy::{Proxy, ProxySchema};
use crate::webserver::requests::HTTPRequest;
use crate::webserver::responses::HTTPResponse;
use crate::webserver::responses::status_code::StatusCode;
use crate::webserver::route::{Route, RouteType};
use log::{error, warn};
use rustls::{ServerConfig, ServerConnection};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Represents a client connected to the webserver.
///
/// The `Client` struct handles reading from the TCP stream (or TLS stream if configured),
/// parsing HTTP requests, applying middleware, routing to the appropriate handler,
/// and sending HTTP responses.
pub(crate) struct Client {
    /// The TCP stream connected to the client.
    stream: TcpStream,
    /// Map of domains to their route configurations.
    domains: Arc<Mutex<HashMap<Domain, Arc<Mutex<Vec<Route>>>>>>,
    /// Default domain used when no matching domain is found.
    default_domain: Domain,
    /// Middleware to apply for requests and responses.
    middleware: Arc<Vec<Middleware>>,
    /// Optional TLS configuration.
    tls_config: Option<Arc<ServerConfig>>,
    /// Optional active TLS connection.
    tls_connection: Option<ServerConnection>,
}

impl Client {
    /// Creates a new `Client` instance.
    ///
    /// # Arguments
    /// * `stream` - The TCP stream for this client.
    /// * `domains` - Shared map of domain routes.
    /// * `default_domain` - Default domain for unmatched requests.
    /// * `middleware` - Middleware to apply.
    /// * `tls_config` - Optional TLS server configuration.
    pub(crate) fn new(
        stream: TcpStream,
        domains: Arc<Mutex<HashMap<Domain, Arc<Mutex<Vec<Route>>>>>>,
        default_domain: Domain,
        middleware: Arc<Vec<Middleware>>,
        tls_config: Option<Arc<ServerConfig>>,
    ) -> Self {
        Self {
            stream,
            domains,
            default_domain,
            middleware,
            tls_config,
            tls_connection: None,
        }
    }

    /// Handles a single client request.
    ///
    /// Reads the HTTP/TLS request, applies middleware, routes it, and sends the response.
    ///
    /// # Arguments
    /// * `i` - Connection iteration (0 if first request, used for TLS handshake).
    ///
    /// # Returns
    /// Returns `Some(ConnectionType)` to indicate whether the connection should
    /// be kept alive, or `None` if the connection closed or an error occurred.
    pub(crate) fn handle(&mut self, i: u32) -> Option<ConnectionType> {
        let raw_request = if self.tls_config.is_some() && i == 0 {
            self.handle_tls_connection()?
        } else {
            self.read_http_request()?
        };

        let request = match HTTPRequest::parse(raw_request.as_ref()) {
            Ok(req) => req,
            Err(_) => {
                error!("Failed to parse HTTP request");
                return None;
            }
        };

        let connection = request.headers().connection.clone();
        let modified_request = self.apply_request_middleware(request.clone());
        let response = self.handle_routing(modified_request);
        let final_response = self.apply_response_middleware(request, response);

        self.send_response(final_response);

        Some(connection)
    }

    /// Reads an HTTP request from the TCP stream.
    ///
    /// Handles reading headers and body based on `Content-Length`.
    fn read_http_request(&mut self) -> Option<String> {
        let _ = self
            .stream
            .set_read_timeout(Some(Duration::from_millis(500)));

        let mut buffer = Vec::with_capacity(2048);
        let mut chunk = [0u8; 1024];
        let mut headers_end_pos = 0;

        loop {
            match self.stream.read(&mut chunk) {
                Ok(0) => return None,
                Ok(n) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                        headers_end_pos = pos + 4;
                        break;
                    }
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(e) => {
                    warn!("Socket read error: {e}");
                    return None;
                }
            }
        }

        let headers_str = String::from_utf8_lossy(&buffer[..headers_end_pos]);
        let content_length: usize = headers_str
            .lines()
            .find(|l| l.to_lowercase().starts_with("content-length:"))
            .and_then(|l| l.split(':').nth(1))
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(0);

        while buffer.len() < headers_end_pos + content_length {
            match self.stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    break;
                }
                Err(e) => {
                    warn!("Failed to read body: {e}");
                    break;
                }
            }
        }

        Some(String::from_utf8_lossy(&buffer).into())
    }

    /// Handles TLS connections, performing handshake and reading initial request.
    fn handle_tls_connection(&mut self) -> Option<String> {
        let tls_cfg = self.tls_config.as_ref()?.clone();
        let mut conn = self.perform_tls_handshake(tls_cfg)?;
        let buffer = self.read_tls_data(&mut conn)?;
        self.tls_connection = Some(conn);
        Some(String::from_utf8_lossy(&buffer).to_string())
    }

    /// Performs a TLS handshake and returns a `ServerConnection`.
    fn perform_tls_handshake(&mut self, tls_config: Arc<ServerConfig>) -> Option<ServerConnection> {
        let mut conn = ServerConnection::new(tls_config).ok()?;
        while conn.is_handshaking() {
            if conn.complete_io(&mut self.stream).is_err() {
                return None;
            }
        }
        Some(conn)
    }

    /// Reads plaintext data from an established TLS connection.
    fn read_tls_data(&mut self, conn: &mut ServerConnection) -> Option<Vec<u8>> {
        let _ = self.stream.set_nonblocking(true);
        let mut buffer = Vec::with_capacity(2048);
        let mut chunk = [0u8; 2048];

        loop {
            if conn.complete_io(&mut self.stream).is_err() {
                return None;
            }

            match conn.reader().read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => return None,
            }

            if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }

        if buffer.is_empty() {
            None
        } else {
            Some(buffer)
        }
    }

    /// Applies request middleware in order for this request.
    fn apply_request_middleware(&self, mut request: HTTPRequest) -> HTTPRequest {
        for middleware in self.middleware.iter() {
            if middleware.route.as_str() != request.path && middleware.route.as_str() != "*" {
                continue;
            }
            if middleware.domain.as_str() != "*"
                && middleware.domain.as_str() != request.host().unwrap_or_default()
            {
                continue;
            }

            match &middleware.f {
                MiddlewareFn::HTTPRequest(func) => func(&mut request),
                MiddlewareFn::Both(req_func, _) => request = req_func(request),
                _ => {}
            }
        }
        request
    }

    /// Applies response middleware in order for this response.
    fn apply_response_middleware(
        &self,
        mut original_request: HTTPRequest,
        mut response: HTTPResponse,
    ) -> HTTPResponse {
        for middleware in self.middleware.iter() {
            match &middleware.f {
                MiddlewareFn::HTTPResponse(func) => func(&mut response),
                MiddlewareFn::BothHTTPResponse(func) => {
                    response = func(&mut original_request, response)
                }
                MiddlewareFn::Both(_, res_func) => response = res_func(response),
                MiddlewareFn::HTTPResponseBothWithRoutes(func) => {
                    response = func(
                        &mut original_request,
                        response,
                        &*self
                            .domains
                            .lock()
                            .unwrap()
                            .get(&self.default_domain)
                            .unwrap()
                            .lock()
                            .unwrap(),
                    )
                }
                _ => {}
            }
        }
        response
    }

    /// Sends an HTTP response to the client, over TLS if applicable.
    fn send_response(&mut self, response: HTTPResponse) {
        let response_bytes = response.to_bytes();

        if let Some(conn) = &mut self.tls_connection {
            let chunk_size = 4096;
            let mut offset = 0;

            while offset < response_bytes.len() {
                let end = (offset + chunk_size).min(response_bytes.len());
                if conn
                    .writer()
                    .write_all(&response_bytes[offset..end])
                    .is_err()
                {
                    warn!("Error writing to TLS stream");
                    return;
                }
                if conn.complete_io(&mut self.stream).is_err() {
                    warn!("Error completing TLS write");
                    return;
                }
                offset = end;
            }

            while conn.wants_write() {
                if conn.complete_io(&mut self.stream).is_err() {
                    break;
                }
            }
        } else {
            let _ = self.stream.write_all(&response_bytes);
            let _ = self.stream.flush();
        }
    }

    /// Routes the HTTP request to the appropriate handler.
    ///
    /// Handles static files, custom routes, proxy routes, and error routes.
    fn handle_routing(&mut self, request: HTTPRequest) -> HTTPResponse {
        let host = request.host().unwrap_or_default();
        let current_domain = Domain::new(&host);

        let guard = self.domains.lock().unwrap();
        let routes_mutex = guard
            .get(&current_domain)
            .or_else(|| guard.get(&self.default_domain));

        let Some(routes_mutex) = routes_mutex else {
            return HTTPResponse::not_found();
        };

        let routes = routes_mutex.lock().unwrap();

        // Longest prefix match
        let matched_prefix = routes
            .iter()
            .filter(|r| request.path.starts_with(&r.route) && r.method == request.method)
            .max_by_key(|r| r.route.len());

        let route = match matched_prefix {
            Some(r) => r,
            None => return HTTPResponse::not_found(),
        };

        let exact = routes
            .iter()
            .find(|r| r.route == request.path)
            .unwrap_or(route);

        if exact.method != request.method {
            return HTTPResponse::method_not_allowed();
        }

        match exact.route_type {
            RouteType::Static => {
                if let Some(folder) = &exact.folder {
                    return get_static_file_response(folder, &request);
                }
            }
            RouteType::File => {
                if let Some(content) = &exact.content {
                    let mut response = HTTPResponse::new(exact.status_code);
                    response.set_body_string(content.to_string());
                    return response;
                }
            }
            RouteType::Custom => {
                if let Some(f) = &exact.f {
                    return catch_unwind(AssertUnwindSafe(|| f(request, &exact.domain)))
                        .unwrap_or_else(|_| HTTPResponse::internal_error());
                }
            }
            RouteType::Proxy => {
                if let Some(external) = &exact.external {
                    return get_proxy_route(&exact.route, external, &request);
                }
            }
            RouteType::Error => {
                if let Some(content) = &exact.content {
                    let mut response = HTTPResponse::new(exact.status_code);
                    response.set_body_string(content.to_string());
                    return response;
                }
            }
        }

        HTTPResponse::internal_error()
    }
}

/// Helper: Handles proxy routes.
fn get_proxy_route(prefix: &str, external: &String, request: &HTTPRequest) -> HTTPResponse {
    let path = format!(
        "{}/{}",
        prefix.trim_end_matches('/'),
        request.path.strip_prefix(prefix).unwrap_or("")
    );
    let joined = if external.ends_with('/') {
        format!("{}{}", external.trim_end_matches('/'), path)
    } else {
        format!("{}{}", external, path)
    };
    let mut proxy = Proxy::new(joined);

    if proxy.parse_url().is_none() {
        return HTTPResponse::bad_gateway();
    }

    let Some(mut stream) = Proxy::connect_to_server(&proxy.host, proxy.port) else {
        return HTTPResponse::bad_gateway();
    };

    let response_data = match proxy.scheme {
        ProxySchema::HTTP => Proxy::send_http_request(&mut stream, &proxy.path, &proxy.host),
        ProxySchema::HTTPS => Proxy::send_https_request(&mut stream, &proxy.path, &proxy.host),
    };

    if let Some(raw_response) = response_data {
        let (body_bytes, content_type) = Proxy::parse_http_response_bytes(&raw_response);
        let mut response = HTTPResponse::new(StatusCode::Ok);
        response.set_body(body_bytes);
        response.message.headers.content_type =
            ContentType::from_str(&*content_type).expect("Could not parse Content-Type");

        response.message.headers.apply_cors_permissive();

        return response;
    }

    HTTPResponse::bad_gateway()
}

/// Helper: Handles static file routes.
fn get_static_file_response(folder: &String, request: &HTTPRequest) -> HTTPResponse {
    let (content, content_type) = get_static_file_content(&request.path, folder);

    if content.is_empty() {
        return HTTPResponse::not_found();
    }

    let mut response = HTTPResponse::ok();
    response.set_body_string(content.to_string());
    response.message.headers.content_type = content_type;
    response
}
