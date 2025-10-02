use crate::webserver::files::get_static_file_content;
use crate::webserver::middleware::{Middleware, MiddlewareFn};
use crate::webserver::requests::Request;
use crate::webserver::responses::Response;
use crate::webserver::responses::ResponseCodes;
use crate::webserver::{Domain, DomainRoutes};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::string::ToString;
use std::sync::{Arc, Mutex};

pub struct Client {
    stream: TcpStream,
    domains: Arc<Mutex<HashMap<Domain, DomainRoutes>>>,
    middleware: Arc<Vec<Middleware>>,
}

impl Client {
    pub fn new(
        stream: TcpStream,
        domains: Arc<Mutex<HashMap<Domain, DomainRoutes>>>,
        middleware: Arc<Vec<Middleware>>,
    ) -> Self {
        Self {
            stream,
            domains,
            middleware,
        }
    }

    pub fn handle(&mut self) {
        let mut buffer = [0u8; 1024];

        let bytes_read = match self.stream.read(&mut buffer) {
            Ok(0) => return,
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to read from socket: {e}");
                return;
            }
        };

        let raw_request = String::from_utf8_lossy(&buffer[..bytes_read]);

        let request = match Request::new(raw_request.to_string()) {
            Some(req) => req,
            None => {
                eprintln!("Failed to parse request");
                return;
            }
        };

        let mut modified_request = request.clone();
        for middleware in self.middleware.iter() {
            match &middleware.f {
                MiddlewareFn::Request(func) => {
                    func(&mut modified_request);
                }
                MiddlewareFn::Both(req_func, _res_func) => {
                    modified_request = req_func(modified_request);
                }
                _ => continue,
            }
        }

        let mut original_request = request.clone();
        let response = self.handle_routing(modified_request);

        let final_response = if !self.middleware.is_empty() {
            let mut resp = response;
            for middleware in self.middleware.iter() {
                match &middleware.f {
                    MiddlewareFn::Response(func) => {
                        func(&mut resp);
                    }
                    MiddlewareFn::BothResponse(func) => {
                        resp = func(&mut original_request, resp);
                    }
                    MiddlewareFn::Both(_req_func, res_func) => {
                        resp = res_func(resp);
                    }
                    _ => continue,
                }
            }
            resp
        } else {
            response
        };

        self.send_message(final_response)
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
                // Custom Route
                handler(request)
            } else if let Some((prefix, folder)) =
                find_static_folder(&domain_routes.static_routes, &request.route)
            {
                // Static Files
                let (content, content_type) = get_static_file_content(&request.route, folder);
                let mut response = Response::new(content, None, None);
                response.headers.add_header("Content-Type", &content_type);
                response
            } else if let Some(resp) = domain_routes.routes.get(&request.route) {
                // Files
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

    fn send_message(&mut self, message: Response) {
        let _ = self.stream.write_all(message.as_str().as_bytes());
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
