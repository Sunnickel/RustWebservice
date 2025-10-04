use crate::webserver::files::get_static_file_content;
use crate::webserver::middleware::{Middleware, MiddlewareFn};
use crate::webserver::requests::Request;
use crate::webserver::responses::Response;
use crate::webserver::responses::ResponseCodes;
use crate::webserver::{Domain, DomainRoutes};
use log::{error, warn};
use rustls::{ServerConfig, ServerConnection};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::string::ToString;
use std::sync::{Arc, Mutex};

pub struct Client {
    stream: TcpStream,
    domains: Arc<Mutex<HashMap<Domain, DomainRoutes>>>,
    middleware: Arc<Vec<Middleware>>,
    tls_config: Option<Arc<ServerConfig>>,
    tls_connection: Option<ServerConnection>,
}

impl Client {
    pub fn new(
        stream: TcpStream,
        domains: Arc<Mutex<HashMap<Domain, DomainRoutes>>>,
        middleware: Arc<Vec<Middleware>>,
        tls_config: Option<Arc<ServerConfig>>,
    ) -> Self {
        Self {
            stream,
            domains,
            middleware,
            tls_config,
            tls_connection: None,
        }
    }

    pub fn handle(&mut self) {
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

    fn handle_tls_connection(&mut self) -> Option<String> {
        let tls_cfg = self.tls_config.as_ref()?.clone();
        let mut conn = self.perform_tls_handshake(tls_cfg)?;

        let buffer = self.read_tls_data(&mut conn)?;

        self.tls_connection = Some(conn);

        Some(String::from_utf8_lossy(&buffer).to_string())
    }

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

    fn apply_request_middleware(&self, mut request: Request) -> Request {
        for middleware in self.middleware.iter() {
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
        request
    }

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
                _ => continue,
            }
        }
        response
    }

    fn send_response(&mut self, response: Response) {
        let response_str = response.as_str();
        let response_bytes = response_str.as_bytes();

        if let Some(conn) = &mut self.tls_connection {
            if let Err(e) = conn.writer().write_all(response_bytes) {
                warn!("Error writing to TLS stream: {}", e);
                return;
            }
            if let Err(e) = conn.complete_io(&mut self.stream) {
                warn!("Error completing TLS write: {}", e);
            }
        } else {
            let _ = self.stream.write_all(response_bytes);
        }
    }

    fn handle_routing(&mut self, request: Request) -> Response {
        let host = request.values.get("host").cloned().unwrap_or_default();
        let guard = self.domains.lock().unwrap();

        let domain_name = if !host.is_empty() {
            host.split('.').next().unwrap_or("").to_string()
        } else {
            "".to_string()
        };

        let domain_routes = guard
            .get(&Domain::new(&domain_name))
            .or_else(|| guard.get(&Domain::new("")));

        if let Some(domain_routes) = domain_routes {
            if let Some(handler) = domain_routes.custom_routes.get(request.route.as_str()) {
                handler(request)
            } else if let Some((_, folder)) =
                find_static_folder(&domain_routes.static_routes, &request.route)
            {
                let (content, content_type) = get_static_file_content(&request.route, folder);
                let mut response = Response::new(content, None, None);
                response.headers.add_header("Content-Type", &content_type);
                response
            } else if let Some(resp) = domain_routes.routes.get(&request.route) {
                Response::new(Arc::from(resp.clone()), Some(ResponseCodes::Ok), None)
            } else {
                Response::new(
                    Arc::from("<h1>404 Not Found</h1>".to_string()),
                    Some(ResponseCodes::NotFound),
                    None,
                )
            }
        } else {
            Response::new(
                Arc::from("<h1>404 Domain Not Found</h1>".to_string()),
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
