use crate::webserver::files::get_static_file_content;
use crate::webserver::middleware::{Middleware, MiddlewareFn};
use crate::webserver::proxy::{Proxy, ProxySchema, find_header_end};
use crate::webserver::requests::Request;
use crate::webserver::responses::Response;
use crate::webserver::responses::ResponseCodes;
use crate::webserver::responses::ResponseCodes::Processing;
use crate::webserver::{Domain, DomainRoutes};
use log::warn;
use rustls::{ServerConfig, ServerConnection};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::panic::{AssertUnwindSafe, catch_unwind};
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
    domains: Arc<Mutex<HashMap<Domain, DomainRoutes>>>,
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
        domains: Arc<Mutex<HashMap<Domain, DomainRoutes>>>,
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
    /// This method reads the raw HTTP or TLS request, parses it into a `Request`,
    /// applies middleware to the request and response, routes the request to
    /// the appropriate handler, and sends back the final response.
    ///
    /// # Side Effects
    ///
    /// - Reads data from the TCP stream.
    /// - Writes response data to the TCP stream.
    /// - May establish a TLS connection if `tls_config` is set.
    pub(crate) fn handle(&mut self) {
        let raw_request = if self.tls_config.is_some() {
            match self.handle_tls_connection() {
                Some(req) => req,
                None => return,
            }
        } else {
            match self.read_http_request() {
                Some(req) => req,
                None => return,
            }
        };
        let request = match Request::new(raw_request, self.stream.peer_addr().unwrap().to_string())
        {
            Some(req) => req,
            None => {
                warn!("Failed to parse request");
                return;
            }
        };
        let modified_request = self.apply_request_middleware(request.clone());
        let response = self.handle_routing(modified_request);
        let final_response = self.apply_response_middleware(request, response);
        self.send_response(final_response);
    }

    /// Reads an HTTP request from the TCP stream.
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the raw HTTP request data if successful,
    /// or `None` if reading fails or no data is available.
    ///
    /// # Errors
    ///
    /// - May return `None` if reading from the stream fails.
    fn read_http_request(&mut self) -> Option<String> {
        let mut buffer = [0u8; 1024];
        let bytes_read = match self.stream.read(&mut buffer) {
            Ok(0) => return None,
            Ok(n) => n,
            Err(e) => {
                warn!("Failed to read from socket: {e}");
                return None;
            }
        };
        Some(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
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
                        warn!("TLS handshake error: {}", e);
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
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            match conn.complete_io(&mut self.stream) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    warn!("Error reading TLS data: {}", e);
                    return None;
                }
            }
            let mut plaintext = conn.reader();
            match plaintext.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    warn!("Error reading plaintext: {}", e);
                    return None;
                }
            }
            if buffer.len() >= 4 {
                let end = &buffer[buffer.len() - 4..];
                if end == b"\r\n\r\n" {
                    break;
                }
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
    /// * `request` - The incoming `Request` to process with middleware.
    ///
    /// # Returns
    ///
    /// A modified `Request` after all applicable middleware have been applied.
    fn apply_request_middleware(&self, mut request: Request) -> Request {
        for middleware in self.middleware.iter() {
            if middleware.route.as_str() != request.route
                && middleware.route.clone().as_str() != "*"
            {
                continue;
            } else if middleware.domain.as_str() == request.values.get("host").unwrap().as_str() {
                continue;
            } else {
                match &middleware.f {
                    MiddlewareFn::Request(func) => {
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
    /// * `original_request` - The original `Request` that generated this response.
    /// * `response` - The `Response` to process with middleware.
    ///
    /// # Returns
    ///
    /// A modified `Response` after all applicable middleware have been applied.
    fn apply_response_middleware(
        &self,
        mut original_request: Request,
        mut response: Response,
    ) -> Response {
        if self.middleware.is_empty() {
            return response;
        }
        for middleware in self.middleware.iter() {
            match &middleware.f {
                MiddlewareFn::Response(func) => {
                    func(&mut response);
                }
                MiddlewareFn::BothResponse(func) => {
                    response = func(&mut original_request, response);
                }
                MiddlewareFn::Both(_, res_func) => {
                    response = res_func(response);
                }
                MiddlewareFn::ResponseBothWithRoutes(func) => {
                    response = func(
                        &mut original_request,
                        response,
                        self.domains
                            .lock()
                            .unwrap()
                            .get(&self.default_domain)
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
    /// * `response` - The `Response` to send to the client.
    ///
    /// # Side Effects
    ///
    /// - Writes data to the TCP stream.
    /// - May write to a TLS stream if a TLS connection is active.
    ///
    /// # Errors
    ///
    /// - May fail to write to the stream, logging a warning but not returning an error.
    fn send_response(&mut self, response: Response) {
        let response_str = response.as_str();
        let response_bytes = response_str.as_bytes();

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
            let _ = self.stream.write_all(response_bytes);
        }
    }

    /// Routes the request to the appropriate handler based on domain and route.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming `Request` to route.
    ///
    /// # Returns
    ///
    /// A `Response` generated by the matched handler or a default 500 error if no handler is found.
    fn handle_routing(&mut self, request: Request) -> Response {
        let host = request.values.get("host").cloned().unwrap_or_default();
        let current_domain = Domain::new(&host);
        let guard = self.domains.lock().unwrap();
        let domain_routes = guard
            .get(&current_domain)
            .or_else(|| guard.get(&self.default_domain));
        if let Some(domain_routes) = domain_routes {
            if let Some(handler) = domain_routes.custom_routes.get(request.route.as_str()) {
                catch_unwind(AssertUnwindSafe(|| handler(request, &current_domain))).unwrap_or_else(
                    |_| {
                        Response::new(
                            Arc::from("<h1>500 Internal Server Error</h1>".to_string()),
                            Some(ResponseCodes::InternalServerError),
                            None,
                        )
                    },
                )
            } else if let Some((_, folder)) =
                find_static_folder(&domain_routes.static_routes, &request.route)
            {
                let (content, content_type) = get_static_file_content(&request.route, folder);

                if content == Arc::from(String::new()) {
                    return Response::new(
                        Arc::from("<h1>404 Not found</h1>".to_string()),
                        Some(ResponseCodes::NotFound),
                        None,
                    );
                }

                let mut response = Response::new(content, None, None);
                response.headers.add_header("Content-Type", &content_type);
                response
            } else if let Some((prefix, external)) =
                find_proxy_route(&domain_routes.proxy_route, &request.route)
            {
                let path = request.route.strip_prefix(prefix).unwrap_or(&request.route);
                let mut proxy = Proxy::new(format!("{}{}", external, path));

                if proxy.parse_url().is_none() {
                    return Response::new(
                        Arc::from("<h1>502 Bad Gateway - Invalid URL</h1>".to_string()),
                        Some(ResponseCodes::BadGateway),
                        None,
                    );
                }

                let Some(mut stream) = Proxy::connect_to_server(&proxy.host, proxy.port) else {
                    return Response::new(
                        Arc::from("<h1>502 Bad Gateway - Connection Failed</h1>".to_string()),
                        Some(ResponseCodes::BadGateway),
                        None,
                    );
                };

                let response_data = match proxy.scheme {
                    ProxySchema::HTTP => {
                        Proxy::send_http_request(&mut stream, &proxy.path, &proxy.host)
                    }
                    ProxySchema::HTTPS => {
                        Proxy::send_https_request(&mut stream, &proxy.path, &proxy.host)
                    }
                };

                if let Some(raw_response) = response_data {
                    println!("=== RAW RESPONSE DEBUG ===");
                    println!("Total bytes received: {}", raw_response.len());

                    // Check for header separator
                    if let Some(header_end) = find_header_end(&raw_response) {
                        println!("Headers end at byte: {}", header_end);

                        let headers = String::from_utf8_lossy(&raw_response[..header_end]);
                        println!("Headers:\n{}", headers);

                        let body_bytes = &raw_response[header_end + 4..];
                        println!("Body length: {}", body_bytes.len());
                        println!(
                            "First 100 body bytes as hex: {:02x?}",
                            &body_bytes[..body_bytes.len().min(100)]
                        );
                        println!(
                            "First 100 body bytes as string: {}",
                            String::from_utf8_lossy(&body_bytes[..body_bytes.len().min(100)])
                        );
                    } else {
                        println!("NO HEADER SEPARATOR FOUND!");
                        println!(
                            "First 500 bytes: {}",
                            String::from_utf8_lossy(&raw_response[..raw_response.len().min(500)])
                        );
                    }
                    println!("=== END DEBUG ===");
                    let (body_bytes, content_type) =
                        Proxy::parse_http_response_bytes(&raw_response);
                    println!("Body bytes length: {}", body_bytes.len());
                    println!(
                        "First 200 bytes: {:?}",
                        &body_bytes[..body_bytes.len().min(200)]
                    );
                    let body_string = String::from_utf8_lossy(&body_bytes).to_string();
                    let mut response =
                        Response::new(Arc::from(body_string), Some(ResponseCodes::Ok), None);
                    response.headers.add_header("Content-Type", &content_type);
                    return response;
                }

                Response::new(
                    Arc::from("<h1>502 Bad Gateway</h1>".to_string()),
                    Some(ResponseCodes::BadGateway),
                    None,
                )
            } else if let Some(resp) = domain_routes.routes.get(&request.route) {
                Response::new(Arc::from(resp.clone()), Some(ResponseCodes::Ok), None)
            } else {
                // Return a 404 response if no route is found.
                Response::new(
                    Arc::from("<h1>404 Not Found</h1>".to_string()),
                    Some(ResponseCodes::NotFound),
                    None,
                )
            }
        } else {
            // Return a 404 response if no domain routes are found.
            Response::new(
                Arc::from("<h1>404 Not Found</h1>".to_string()),
                Some(ResponseCodes::NotFound),
                None,
            )
        }
    }
}

fn find_static_folder<'a>(
    static_routes: &'a HashMap<String, String>,
    route: &str,
) -> Option<(&'a str, &'a String)> {
    static_routes
        .iter()
        .filter(|(prefix, _)| route.starts_with(prefix.as_str()))
        .max_by_key(|(prefix, _)| prefix.len())
        .map(|(prefix, folder)| (prefix.as_str(), folder))
}

fn find_proxy_route<'a>(
    proxy_routes: &'a HashMap<String, String>,
    route: &str,
) -> Option<(&'a str, &'a String)> {
    proxy_routes
        .iter()
        .filter(|(prefix, _)| route.starts_with(prefix.as_str()))
        .max_by_key(|(prefix, _)| prefix.len())
        .map(|(prefix, url)| (prefix.as_str(), url))
}
