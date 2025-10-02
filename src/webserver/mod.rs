mod client_handling;
mod files;
mod middleware;
pub(crate) mod requests;
pub(crate) mod responses;

use crate::webserver::client_handling::Client;
use crate::webserver::files::get_file_content;
use crate::webserver::middleware::Middleware;
use crate::webserver::requests::Request;
use crate::webserver::responses::Response;
use chrono::Utc;
use log::{info, trace};
use std::collections::HashMap;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Domain {
    pub name: String,
}

impl Domain {
    pub fn new(name: &str) -> Domain {
        Self {
            name: name.to_string(),
        }
    }

    pub fn from(name: &str) -> Domain {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct DomainRoutes {
    pub routes: HashMap<String, String>,
    pub static_routes: HashMap<String, String>,
    pub custom_routes: HashMap<String, Arc<dyn Fn(Request) -> Response + Send + Sync>>,
}

impl DomainRoutes {
    pub fn new() -> DomainRoutes {
        Self {
            routes: HashMap::new(),
            static_routes: HashMap::new(),
            custom_routes: HashMap::new(),
        }
    }
}

pub struct WebServer {
    pub(crate) host: [u8; 4],
    pub port: u16,
    pub domains: Arc<Mutex<HashMap<Domain, DomainRoutes>>>,
    pub default_domain: Domain,
    pub middleware: Arc<Vec<Middleware>>,
}

impl WebServer {
    pub fn new(host: [u8; 4], port: u16) -> WebServer {
        let mut domains = HashMap::new();
        let default_domain = Domain::new("");
        domains.insert(default_domain.clone(), DomainRoutes::new());

        let mut middlewares = Vec::new();
        let logging_middleware = Middleware::new_response_both(None, None, Self::logging);

        middlewares.push(logging_middleware);

        WebServer {
            host,
            port,
            domains: Arc::new(Mutex::new(domains)),
            middleware: Arc::from(middlewares),
            default_domain,
        }
    }

    pub fn start(&self) {
        let bind_addr = format!(
            "{}.{}.{}.{}:{}",
            self.host[0], self.host[1], self.host[2], self.host[3], self.port
        );

        let listener = TcpListener::bind(&bind_addr).unwrap();
        info!("Server running on http://{bind_addr}/");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let domains = Arc::clone(&self.domains);
                    let middleware = Arc::clone(&self.middleware);
                    thread::spawn(move || {
                        let mut client: Client = Client::new(stream, domains, middleware);
                        client.handle();
                    });
                }
                Err(e) => eprintln!("Connection failed: {e}"),
            }
        }
    }

    pub fn add_subdomain_router(&mut self, mut domain: Domain) {
        let mut guard = self.domains.lock().unwrap();
        domain.name = domain.name.to_lowercase();
        guard.entry(domain).or_insert_with(DomainRoutes::new);
    }

    pub fn add_route_file(&mut self, route: &str, file_path: &str, mut domain: Option<Domain>) {
        if domain.is_none() {
            domain = Some(self.default_domain.clone());
        }

        let content = get_file_content(&PathBuf::from(file_path));

        let domain_key: String = domain.unwrap().name.to_string();
        let mut guard = self.domains.lock().unwrap();
        if let Some(domain_routes) = guard.get_mut(&Domain::new(&*domain_key)) {
            domain_routes
                .routes
                .insert(route.to_string(), content.to_string());
        } else {
            panic!("Domain not found: {}", domain_key);
        }
    }

    pub fn add_static_route(&mut self, route: &str, folder: &str, mut domain: Option<Domain>) {
        if domain.is_none() {
            domain = Some(self.default_domain.clone());
        }

        let folder_path = PathBuf::from(folder);
        if !folder_path.exists() {
            panic!("Folder doesn't exist: {}", folder);
        }

        let domain_key: String = domain.unwrap().name.to_string();
        let mut guard = self.domains.lock().unwrap();
        if let Some(domain_routes) = guard.get_mut(&Domain::new(&*domain_key)) {
            domain_routes
                .static_routes
                .insert(route.to_string(), folder.to_string());
        } else {
            panic!("Domain not found: {}", domain_key);
        }
    }

    pub fn add_custom_route(
        &self,
        route: &str,
        f: impl Fn(Request) -> Response + Send + Sync + 'static,
        domain: Option<Domain>,
    ) {
        let domain: Domain = domain.unwrap_or_else(|| Domain::new(""));

        let mut guard = self.domains.lock().unwrap();

        if let Some(domain_routes) = guard.get_mut(&domain) {
            domain_routes
                .custom_routes
                .insert(route.to_string(), Arc::new(f));
        } else {
            panic!("Domain not found: {}", domain.name);
        }
    }

    pub fn logging(request: &mut Request, response: Response) -> Response {
        trace!(
            "[{}] {:<6} {:<30} -> {:3} (host: {})",
            Utc::now().format("%Y-%m-%d %H:%M:%S"),
            request.method,
            request.route,
            response.headers.get_status_code(),
            request
                .values
                .get("host")
                .unwrap_or(&"<unknown>".to_string())
        );
        response
    }
}
