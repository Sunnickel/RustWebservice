use crate::webserver::files::get_static_file_content;
use crate::webserver::http_packet::header::connection::ConnectionType;
use crate::webserver::http_packet::header::content_types::ContentType;
use crate::webserver::middleware::{Middleware, MiddlewareFn};
use crate::webserver::proxy::{Proxy, ProxySchema};
use crate::webserver::requests::HTTPRequest;
use crate::webserver::responses::status_code::StatusCode;
use crate::webserver::responses::HTTPResponse;
use crate::webserver::route::{Route, RouteType};
use crate::webserver::Domain;
use log::{error, warn};
use rustls::{ServerConfig, ServerConnection};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, Mutex};

/// Represents a client connection to the webserver.
///
/// This struct holds the TCP stream, domain routing information,
/// middleware, and TLS configuration for handling requests.
pub(crate) struct Client {
    /// The underlying TCP stream for communication with the client.
    stream: TcpStream,
    /// A map of domains to their route configurations, shared across threads.
    domains: Arc<Mutex<HashMap<Domain, Arc<Mutex<Vec<Route>>>>>>,
    /// The default domain used when no matching domain is found.
    default_domain: Domain,
    /// List of middleware functions to apply to requests and responses.
    middleware: Arc<Vec<Middleware>>,
    /// Optional TLS configuration for secure connections.
    tls_config: Option<Arc<ServerConfig>>,
    /// The active TLS connection, if established.
    tls_connection: Option<ServerConnection>,
}

impl Client {
    /// Creates a new `Client` instance.
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream for communication with the client.
    /// * `domains` - Shared map of domains to their route configurations.
    /// * `default_domain` - The default domain used when no match is found.
    /// * `middleware` - List of middleware functions to apply.
    /// * `tls_config` - Optional TLS configuration for secure connections.
    ///
    /// # Returns
    ///
    /// A new `Client` instance initialized with the provided parameters.
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

    /// Handles an incoming client request.
    ///
    /// This method reads the raw HTTP or TLS request, parses it into a `HTTPRequest`,
    /// applies middleware to the request and response, routes the request to
    /// the appropriate handler, and sends back the final response.
    ///
    /// # Side Effects
    ///
    /// - Reads data from the TCP stream.
    /// - Writes response data to the TCP stream.
    /// - May establish a TLS connection if `tls_config` is set.
    pub(crate) fn handle(&mut self, i: u32) -> Option<ConnectionType> {
        let raw_request = if self.tls_config.is_some() && i <= 0 {
            match self.handle_tls_connection() {
                Some(req) => req,
                None => return None,
            }
        } else {
            match self.read_http_request() {
                Some(req) => req,
                None => return None,
            }
        };

        let request = match HTTPRequest::parse(raw_request.as_ref()) {
            Ok(request) => request,
            Err(_) => {
                error!("Could not parse HTTP request!");
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

    fn read_http_request(&mut self) -> Option<String> {
        use std::time::Duration;
        self.stream
            .set_read_timeout(Some(Duration::from_millis(500)));

        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
        let mut headers_complete = false;
        let mut headers_end_pos = 0;

        while !headers_complete {
            match self.stream.read(&mut chunk) {
                Ok(0) => return None, // client closed connection
                Ok(n) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                        headers_complete = true;
                        headers_end_pos = pos + 4;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => break,
                Err(e) => {
                    warn!("Socket read error: {e}");
                    return None;
                }
            }
        }

        if !headers_complete {
            return None;
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

    /// Handles the TLS connection process for secure requests.
    ///
    /// Performs a TLS handshake and reads the initial data sent by the client.
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the raw TLS request data if successful,
    /// or `None` if the handshake or reading fails.
    ///
    /// # Errors
    ///
    /// - May return `None` if TLS handshake fails or data reading fails.
    fn handle_tls_connection(&mut self) -> Option<String> {
        let tls_cfg = self.tls_config.as_ref()?.clone();
        let mut conn = self.perform_tls_handshake(tls_cfg)?;
        let buffer = self.read_tls_data(&mut conn)?;
        self.tls_connection = Some(conn);
        Some(String::from_utf8_lossy(&buffer).to_string())
    }

    /// Performs a TLS handshake using the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `tls_config` - The TLS server configuration to use for the handshake.
    ///
    /// # Returns
    ///
    /// An `Option<ServerConnection>` representing the established TLS connection,
    /// or `None` if the handshake fails.
    ///
    /// # Errors
    ///
    /// - May return `None` if creating or completing the TLS handshake fails.
    fn perform_tls_handshake(&mut self, tls_config: Arc<ServerConfig>) -> Option<ServerConnection> {
        let mut conn = match ServerConnection::new(tls_config) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to create TLS connection: {}", e);
                return None;
            }
        };
        loop {
            if conn.is_handshaking() {
                match conn.complete_io(&mut self.stream) {
                    Ok(_) => {}
                    Err(e) => {
                        return None;
                    }
                }
            } else {
                break;
            }
        }
        Some(conn)
    }

    /// Reads data from a TLS connection.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the TLS connection to read from.
    ///
    /// # Returns
    ///
    /// An `Option<Vec<u8>>` containing the raw TLS data if successful,
    /// or `None` if reading fails.
    ///
    /// # Errors
    ///
    /// - May return `None` if reading from the TLS stream fails.
    fn read_tls_data(&mut self, conn: &mut ServerConnection) -> Option<Vec<u8>> {
        use std::io::Read;

        let _ = self.stream.set_nonblocking(true);

        let mut buffer = Vec::new();
        let mut chunk = [0u8; 2048];

        loop {
            match conn.complete_io(&mut self.stream) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(_) => return None,
            }

            let mut plaintext = conn.reader();
            match plaintext.read(&mut chunk) {
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

    /// Applies request middleware to the incoming request.
    ///
    /// Middleware functions are executed in order, and applied based on route and domain match.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming `HTTPRequest` to process with middleware.
    ///
    /// # Returns
    ///
    /// A modified `HTTPRequest` after all applicable middleware have been applied.
    fn apply_request_middleware(&self, mut request: HTTPRequest) -> HTTPRequest {
        for middleware in self.middleware.iter() {
            if middleware.route.as_str() != request.path && middleware.route.clone().as_str() != "*"
            {
                continue;
            } else if middleware.domain.as_str() == request.host().unwrap() {
                continue;
            } else {
                match &middleware.f {
                    MiddlewareFn::HTTPRequest(func) => {
                        func(&mut request);
                    }
                    MiddlewareFn::Both(req_func, _) => {
                        request = req_func(request);
                    }
                    _ => continue,
                }
            }
        }
        request
    }

    /// Applies response middleware to the outgoing response.
    ///
    /// # Arguments
    ///
    /// * `original_request` - The original `HTTPRequest` that generated this response.
    /// * `response` - The `HTTPResponse` to process with middleware.
    ///
    /// # Returns
    ///
    /// A modified `HTTPResponse` after all applicable middleware have been applied.
    fn apply_response_middleware(
        &self,
        mut original_request: HTTPRequest,
        mut response: HTTPResponse,
    ) -> HTTPResponse {
        if self.middleware.is_empty() {
            return response;
        }
        for middleware in self.middleware.iter() {
            match &middleware.f {
                MiddlewareFn::HTTPResponse(func) => {
                    func(&mut response);
                }
                MiddlewareFn::BothHTTPResponse(func) => {
                    response = func(&mut original_request, response);
                }
                MiddlewareFn::Both(_, res_func) => {
                    response = res_func(response);
                }
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
                    );
                }
                _ => continue,
            }
        }
        response
    }

    /// Sends a response back to the client.
    ///
    /// # Arguments
    ///
    /// * `response` - The `HTTPResponse` to send to the client.
    ///
    /// # Side Effects
    ///
    /// - Writes data to the TCP stream.
    /// - May write to a TLS stream if a TLS connection is active.
    ///
    /// # Errors
    ///
    /// - May fail to write to the stream, logging a warning but not returning an error.
    fn send_response(&mut self, response: HTTPResponse) {
        let response_bytes = response.to_bytes();

        if let Some(conn) = &mut self.tls_connection {
            let chunk_size = 4096;
            let mut offset = 0;

            while offset < response_bytes.len() {
                let end = (offset + chunk_size).min(response_bytes.len());
                let chunk = &response_bytes[offset..end];

                if let Err(e) = conn.writer().write(chunk) {
                    warn!("Error writing to TLS stream: {}", e);
                    return;
                }

                if let Err(e) = conn.complete_io(&mut self.stream) {
                    warn!("Error completing TLS write: {}", e);
                    return;
                }

                offset = end;
            }

            while conn.wants_write() {
                if let Err(e) = conn.complete_io(&mut self.stream) {
                    warn!("Error in final flush: {}", e);
                    break;
                }
            }
        } else {
            let _ = self.stream.write_all(response_bytes.as_slice());
            let _ = self.stream.flush();
        }
    }

    /// Routes the request to the appropriate handler based on domain and route.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming `HTTPRequest` to route.
    ///
    /// # Returns
    ///
    /// A `HTTPResponse` generated by the matched handler or a default 500 error if no handler is found.
    fn handle_routing(&mut self, request: HTTPRequest) -> HTTPResponse {
        let host = request.host().unwrap();
        let current_domain = Domain::new(&host);

        let guard = self.domains.lock().unwrap();
        let routes_vec = guard
            .get(&current_domain)
            .or_else(|| guard.get(&self.default_domain));

        let Some(routes_mutex) = routes_vec else {
            return HTTPResponse::not_found();
        };

        let routes = routes_mutex.lock().unwrap();

        let matched_prefix = routes.iter().find(|r| {
            request.path.starts_with(&r.route)
                && std::mem::discriminant(&r.method) == std::mem::discriminant(&request.method)
        });

        let route = match matched_prefix {
            Some(r) => r,
            None => return HTTPResponse::not_found(),
        };

        let exact = match routes.iter().find(|r| r.route == request.path) {
            Some(r) => r,
            None => {
                if route.route_type != RouteType::Static {
                    return HTTPResponse::not_found();
                } else {
                    route
                }
            }
        };

        if std::mem::discriminant(&exact.method) != std::mem::discriminant(&request.method) {
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

        // Fallback if nothing matched or invalid route configuration
        HTTPResponse::internal_error()
    }
}

fn get_proxy_route(prefix: &str, external: &String, request: &HTTPRequest) -> HTTPResponse {
    let path = format!(
        "{}/{}",
        prefix.trim_end_matches('/'),
        request.path.strip_prefix(prefix).unwrap_or("")
    );
    let joined = if external.ends_with('/') {
        format!("{}{}", &external.trim_end_matches('/'), path)
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

    let host_header = if proxy.port == 80 || proxy.port == 443 {
        proxy.host.clone()
    } else {
        format!("{}:{}", proxy.host, proxy.port)
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
            ContentType::from_str(&*content_type).expect("Couldnt Parse Content Type!");

        response.message.headers.apply_cors_permissive();

        return response;
    }

    HTTPResponse::bad_gateway()
}

fn get_static_file_response(folder: &String, request: &HTTPRequest) -> HTTPResponse {
    let (content, content_type) = get_static_file_content(&request.path, folder);

    if content == Arc::from(String::new()) {
        return HTTPResponse::not_found();
    }

    let mut response = HTTPResponse::ok();
    response.set_body_string(content.to_string());
    response.message.headers.content_type =
        ContentType::from_str(&*content_type).expect("Could not parse ContentType!");
    response
}
