mod client_handling;
mod files;
mod middleware;
pub(crate) mod requests;
pub(crate) mod responses;

use crate::webserver::client_handling::Client;
use crate::webserver::files::get_file_content;
use crate::webserver::middleware::Middleware;
use crate::webserver::requests::Request;
use log::info;
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
    pub custom_routes: HashMap<String, Arc<dyn Fn(Request) -> String + Send + Sync>>,
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
        let middleware = Middleware::new_both(None, None, move |x| x.clone(), move |x1| x1.clone());

        middlewares.push(middleware);

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
                    thread::spawn(move || {
                        let mut client: Client = Client::new(stream, domains);
                        client.handle();
                    });
                }
                Err(e) => eprintln!("Connection failed: {e}"),
            }
        }
    }

    pub fn add_subdomain_server(&mut self, mut domain: Domain) {
        let mut guard = self.domains.lock().unwrap();
        domain.name = domain.name.to_lowercase();
        guard.entry(domain).or_insert_with(DomainRoutes::new);
    }

    pub fn add_route_file(&mut self, route: &str, file_path: &str, mut domain: Option<Domain>) {
        if domain.is_none() {
            domain = Some(self.default_domain.clone());
        }

        let content = get_file_content(&PathBuf::from(file_path));

        let domain_key = domain.unwrap().name.to_string();
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

        let domain_key = domain.unwrap().name.to_string();
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
        f: impl Fn(Request) -> String + Send + Sync + 'static,
        domain: Option<Domain>,
    ) {
        let domain = domain.unwrap_or_else(|| Domain::new(""));

        let mut guard = self.domains.lock().unwrap();

        if let Some(domain_routes) = guard.get_mut(&domain) {
            domain_routes
                .custom_routes
                .insert(route.to_string(), Arc::new(f));
        } else {
            panic!("Domain not found: {}", domain.name);
        }
    }
}
