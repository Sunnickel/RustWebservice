mod client_handling;
pub(crate) mod files;
mod middleware;
pub(crate) mod requests;
pub(crate) mod responses;
pub(crate) mod server_config;

use crate::webserver::client_handling::Client;
use crate::webserver::files::get_file_content;
use crate::webserver::middleware::Middleware;
use crate::webserver::requests::Request;
use crate::webserver::responses::Response;
use crate::webserver::server_config::ServerConfig;
use chrono::Utc;
use log::{debug, info, trace};
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
    pub config: ServerConfig,
    pub domains: Arc<Mutex<HashMap<Domain, DomainRoutes>>>,
    pub default_domain: Domain,
    pub middleware: Arc<Vec<Middleware>>,
}

impl WebServer {
    pub fn new(config: ServerConfig) -> WebServer {
        let mut domains = HashMap::new();
        let default_domain = Domain::new("");
        domains.insert(default_domain.clone(), DomainRoutes::new());

        let mut middlewares = Vec::new();
        let logging_middleware = Middleware::new_response_both(None, None, Self::logging);

        middlewares.push(logging_middleware);

        WebServer {
            config,
            domains: Arc::new(Mutex::new(domains)),
            default_domain,
            middleware: Arc::from(middlewares),
        }
    }

    pub fn start(&self) {
        let bind_addr = self.config.ip_as_string();
        let listener = TcpListener::bind(&bind_addr).unwrap();

        if self.config.using_https {
            info!("Server running on https://{bind_addr}/");
        } else {
            info!(
                "Server running on http://{bind_addr}/",
                bind_addr = bind_addr
            );
        }

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let domains = Arc::clone(&self.domains);
                    let middleware = Arc::clone(&self.middleware);
                    let tls_config = self.config.tls_config.clone();
                    thread::spawn(move || {
                        let mut client = Client::new(stream, domains, middleware, tls_config);
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

    pub fn add_route_file(
        &mut self,
        route: &str,
        file_path: &str,
        domain: Option<Domain>,
    ) -> Result<(), String> {
        let domain = domain.unwrap_or_else(|| self.default_domain.clone());
        let content = get_file_content(&PathBuf::from(file_path));

        let mut guard = self.domains.lock().unwrap();

        if !guard.contains_key(&domain) {
            guard.insert(domain.clone(), DomainRoutes::new());
        }

        match guard.get_mut(&domain) {
            Some(domain_routes) => {
                domain_routes
                    .routes
                    .insert(route.to_string(), content.to_string());
                Ok(())
            }
            None => Err(format!(
                "Failed to add route file for domain: {}",
                domain.name
            )),
        }
    }

    pub fn add_static_route(
        &mut self,
        route: &str,
        folder: &str,
        domain: Option<Domain>,
    ) -> Result<(), String> {
        let domain = domain.unwrap_or_else(|| self.default_domain.clone());
        let folder_path = PathBuf::from(folder);

        if !folder_path.exists() {
            return Err(format!("Folder doesn't exist: {}", folder));
        }

        let mut guard = self.domains.lock().unwrap();

        if !guard.contains_key(&domain) {
            guard.insert(domain.clone(), DomainRoutes::new());
        }

        match guard.get_mut(&domain) {
            Some(domain_routes) => {
                domain_routes
                    .static_routes
                    .insert(route.to_string(), folder.to_string());
                Ok(())
            }
            None => Err(format!(
                "Failed to add static route for domain: {}",
                domain.name
            )),
        }
    }

    pub fn add_custom_route(
        &self,
        route: &str,
        f: impl Fn(Request) -> Response + Send + Sync + 'static,
        domain: Option<Domain>,
    ) -> Result<(), String> {
        let domain = domain.unwrap_or_else(|| Domain::new(""));

        match self.domains.lock().unwrap().get_mut(&domain) {
            Some(domain_routes) => {
                domain_routes
                    .custom_routes
                    .insert(route.to_string(), Arc::new(f));
                Ok(())
            }
            None => Err(format!("Domain not found: {}", domain.name)),
        }
    }

    pub fn logging(request: &mut Request, response: Response) -> Response {
        info!(
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
