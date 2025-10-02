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

        let mut request: Request =
            Request::new(raw_request.to_string()).expect("Failed to parse request.");
        for middleware in self.middleware.iter() {
            match &middleware.f {
                MiddlewareFn::Request(func) => {
                    func(&mut request);
                }
                MiddlewareFn::Both(req_func, res_func) => {
                    request = req_func(request);
                }
                _ => continue,
            }
        }

        let mut response = self.handle_routing(request.clone());
        for middleware in self.middleware.iter() {
            match &middleware.f {
                MiddlewareFn::Response(func) => {
                    func(&mut response);
                }
                MiddlewareFn::BothResponse(func) => {
                    response = func(&mut request, response);
                }
                MiddlewareFn::Both(req_func, res_func) => {
                    response = res_func(response);
                }
                _ => continue,
            }
        }

        self.send_message(response)
    }

    fn handle_routing(&mut self, request: Request) -> Response {
        let host = request.values.get("host").unwrap();
        let guard = self.domains.lock().unwrap();

        let domain_routes = guard
            .get(&Domain::new(&host.split(".").collect::<Vec<_>>()[0]))
            .or_else(|| guard.get(&Domain::new("")));

        if let Some(domain_routes) = domain_routes {
            if let Some(handler) = domain_routes.custom_routes.get(request.route.as_str()) {
                // Custom Route
                handler(request)
            } else if let Some((_prefix, folder)) =
                find_static_folder(&domain_routes.static_routes, &*request.route)
            {
                // Static Files
                let (content, content_type) = get_static_file_content(&*request.route, folder);
                let mut response = Response::new(content, None, None);
                response.headers.add_header("Content-Type", &content_type);
                response
            } else if let Some(resp) = domain_routes.routes.get(&*request.route) {
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
