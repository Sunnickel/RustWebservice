use crate::webserver::files::get_static_file_content;
use crate::webserver::requests::Request;
use crate::webserver::responses::ResponseCodes;
use crate::webserver::responses::{generate_response, generate_static_response, Response};
use crate::webserver::{Domain, DomainRoutes};
use chrono::Utc;
use log::trace;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::string::ToString;
use std::sync::{Arc, Mutex};

pub struct Client {
    stream: TcpStream,
    domains: Arc<Mutex<HashMap<Domain, DomainRoutes>>>,
}

impl Client {
    pub fn new(stream: TcpStream, domains: Arc<Mutex<HashMap<Domain, DomainRoutes>>>) -> Self {
        Self { stream, domains }
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

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);

        let request: Request = Request::new(request.to_string()).expect("Failed to parse request.");

        print!("{:?}", request.get_cookies());

        let response = self.handle_routing(request);

        if response == "" {
            return;
        }
        self.send_message(response)
    }

    fn handle_routing(&mut self, request: Request) -> String {
        let host = request.values.get("host").unwrap();
        let guard = self.domains.lock().unwrap();

        let domain_routes = guard
            .get(&Domain::new(&host.split(".").collect::<Vec<_>>()[0]))
            .or_else(|| guard.get(&Domain::new("")));

        if let Some(domain_routes) = domain_routes {
            trace!(
                "[{}]: {} {} on {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S"),
                request.method,
                request.route,
                host
            );
            if let Some(handler) = domain_routes.custom_routes.get(request.route.as_str()) {
                // Custom Route
                handler(request)
            } else if let Some((_prefix, folder)) =
                find_static_folder(&domain_routes.static_routes, &*request.route)
            {
                // Static Files
                let (content, content_type) = get_static_file_content(&*request.route, folder);
                generate_static_response(&mut Response::new(content, None, None), &*content_type)
            } else if let Some(resp) = domain_routes.routes.get(&*request.route) {
                // Files
                generate_response(&mut Response::new(
                    Arc::from(resp.clone()),
                    Some(ResponseCodes::Ok),
                    None,
                ))
            } else {
                generate_response(&mut Response::new(
                    Arc::from("<h1>404 Not Found</h1>".to_string()),
                    Some(ResponseCodes::NotFound),
                    None,
                ))
            }
        } else {
            generate_response(&mut Response::new(
                Arc::from("<h1>404 Domain Not Found</h1>".to_string()),
                Some(ResponseCodes::NotFound),
                None,
            ))
        }
    }

    fn send_message(&mut self, message: String) {
        let _ = self.stream.write_all(message.as_bytes());
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
