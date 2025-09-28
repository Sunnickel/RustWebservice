use crate::webserver::files::get_static_file_content;
use crate::webserver::responses::{generate_response, generate_static_response};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::string::ToString;
use std::sync::Arc;

pub struct Client<'a> {
    stream: TcpStream,
    routes: &'a HashMap<String, Arc<dyn Fn() -> String + Send + Sync + 'static>>,
    static_routes: &'a HashMap<String, String>,
}

impl<'a> Client<'a> {
    pub fn new(
        stream: TcpStream,
        routes: &'a HashMap<String, Arc<dyn Fn() -> String + Send + Sync + 'static>>,
        static_routes: &'a HashMap<String, String>,
    ) -> Self {
        Self {
            stream,
            routes,
            static_routes,
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

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let response = self.handle_routing(request.to_string());

        if response == "" {
            return;
        }
        self.send_message(response)
    }

    fn handle_routing(&mut self, request: String) -> String {
        let max = extract_method_and_route(&request);
        println!("{max:?}");

        if let Some((_method, route)) = max {
            if let Some(folder) = find_static_folder(self.static_routes, route) {
                let (content, content_type) = get_static_file_content(&route, folder);
                generate_static_response(&*content, &*content_type)
            } else if let Some(handler) = self.routes.get(route) {
                handler()
            } else {
                generate_response("<h1>404 Not Found</h1>")
            }
        } else {
            "".to_string()
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
