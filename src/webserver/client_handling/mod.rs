use crate::webserver::files::get_static_file_content;
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
        let response = self.handle_routing(request.to_string());

        if response == "" {
            return;
        }
        self.send_message(response)
    }

    fn handle_routing(&mut self, request: String) -> String {
        let (method, route) = match extract_method_and_route(&request) {
            Some(v) => v,
            None => return "".to_string(),
        };

        let host = extract_host(&request);
        let guard = self.domains.lock().unwrap();

        let domain_routes = guard
            .get(&Domain::new(&host))
            .or_else(|| guard.get(&Domain::new("")));

        if let Some(domain_routes) = domain_routes {
            trace!(
                "[{}]: {} {} on {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S"),
                method,
                route,
                host
            );
            if let Some(handler) = domain_routes.custom_routes.get(route) {
                handler(request)
            } else if let Some((_prefix, folder)) =
                find_static_folder(&domain_routes.static_routes, route)
            {
                let (content, content_type) = get_static_file_content(&route, folder);
                generate_static_response(&mut Response::new(content), &*content_type)
            } else if let Some(handler) = domain_routes.routes.get(route) {
                handler()
            } else {
                generate_response(&mut Response::new(Arc::from(
                    "<h1>404 Not Found</h1>".to_string(),
                )))
            }
        } else {
            generate_response(&mut Response::new(Arc::from(
                "<h1>404 Domain Not Found</h1>".to_string(),
            )))
        }
    }

    fn send_message(&mut self, message: String) {
        let _ = self.stream.write_all(message.as_bytes());
    }
}

fn extract_method_and_route(request: &str) -> Option<(&str, &str)> {
    request.lines().next().and_then(|line| {
        let mut parts = line.split_whitespace();
        let method = parts.next()?;
        let route = parts.next()?;
        Some((method, route))
    })
}

fn extract_host(request: &str) -> String {
    for line in request.lines() {
        if line.to_lowercase().starts_with("host:") {
            return line[5..].trim().to_string();
        }
    }
    "".to_string()
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
