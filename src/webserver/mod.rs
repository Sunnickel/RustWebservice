//! A web server implementation with domain routing capabilities.
//!
//! This module provides the core components for building a web server,
//! including domain handling, routing, middleware support, and request/response processing.

mod client_handling;
pub(crate) mod cookie;
pub(crate) mod files;
pub(crate) mod logger;
pub(crate) mod middleware;
mod proxy;
mod requests;
pub mod responses;
pub(crate) mod server_config;

use crate::webserver::client_handling::Client;
use crate::webserver::files::get_file_content;
use crate::webserver::middleware::Middleware;
use crate::webserver::requests::Request;
use crate::webserver::responses::{Response, ResponseCodes};
pub use crate::webserver::server_config::ServerConfig;

use chrono::Utc;
use log::{error, info, warn};
use std::collections::HashMap;
use std::net::{Shutdown, TcpListener};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

/// Represents a domain name used for routing.
#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Domain {
    /// The domain name as a string.
    pub name: String,
}

impl Domain {
    /// Creates a new `Domain` instance.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the domain name.
    ///
    /// # Returns
    ///
    /// A new `Domain` instance with the specified name.
    pub fn new(name: &str) -> Domain {
        Self {
            name: name.to_string(),
        }
    }

    /// Returns the domain name as a string.
    ///
    /// # Returns
    ///
    /// The domain name as a `String`.
    pub fn as_str(&self) -> String {
        self.name.clone()
    }
}

/// Holds routing information for a specific domain.
#[derive(Clone)]
pub(crate) struct DomainRoutes {
    /// Map of route patterns to file content strings.
    pub(crate) routes: HashMap<String, String>,
    /// Map of route patterns to folder paths for static files.
    pub(crate) static_routes: HashMap<String, String>,
    /// Map of route patterns to custom route handlers.
    pub(crate) custom_routes:
        HashMap<String, Arc<dyn Fn(Request, &Domain) -> Response + Send + Sync>>,
    /// Map of routes where to go after an Error or code.
    pub(crate) code_routes: HashMap<ResponseCodes, String>,
    /// The routes on the server where to proxy it too.
    pub(crate) proxy_route: HashMap<String, String>,
}

impl DomainRoutes {
    /// Creates a new `DomainRoutes` instance with empty maps.
    ///
    /// # Returns
    ///
    /// A new `DomainRoutes` instance.
    pub(crate) fn new() -> DomainRoutes {
        Self {
            routes: HashMap::new(),
            static_routes: HashMap::new(),
            custom_routes: HashMap::new(),
            code_routes: HashMap::new(),
            proxy_route: HashMap::new(),
        }
    }
}

/// The main web server structure.
pub struct WebServer {
    /// Server configuration including IP, port, and TLS settings.
    pub(crate) config: ServerConfig,
    /// Map of domains to their respective routing configurations.
    pub(crate) domains: Arc<Mutex<HashMap<Domain, DomainRoutes>>>,
    /// The default domain used for subdomain generation.
    pub(crate) default_domain: Domain,
    /// List of middleware functions to be applied to requests and responses.
    pub(crate) middleware: Arc<Vec<Middleware>>,
}

impl WebServer {
    /// Creates a new `WebServer` instance with the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - A `ServerConfig` struct containing server settings.
    ///
    /// # Returns
    ///
    /// A new `WebServer` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use webserver::ServerConfig;
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let server = WebServer::new(config);
    /// ```
    pub fn new(config: ServerConfig) -> WebServer {
        let mut domains = HashMap::new();
        let default_domain = Domain::new(&*config.base_domain);
        domains.insert(default_domain.clone(), DomainRoutes::new());
        let mut middlewares = Vec::new();
        let logging_middleware = Middleware::new_response_both(None, None, Self::logging);
        let error_page_middleware =
            Middleware::new_response_both_w_routes(None, None, Self::errorpage);

        middlewares.push(logging_middleware);
        middlewares.push(error_page_middleware);

        WebServer {
            config,
            domains: Arc::new(Mutex::new(domains)),
            default_domain,
            middleware: Arc::from(middlewares),
        }
    }

    /// Starts the web server and begins listening for incoming connections.
    ///
    /// # Side Effects
    ///
    /// * Binds to the configured IP address and port.
    /// * Spawns new threads for each incoming connection.
    /// * Logs server start information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use webserver::ServerConfig;
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let mut server = WebServer::new(config);
    /// // Note: This will block the calling thread
    /// server.start();
    /// ```
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
                    let default_domain = self.default_domain.clone();
                    thread::spawn(move || {
                        let mut client =
                            Client::new(stream, domains, default_domain, middleware, tls_config);
                        client.handle();
                    });
                }
                Err(e) => eprintln!("Connection failed: {e}"),
            }
        }
    }

    /// Adds a new subdomain router for the specified domain.
    ///
    /// # Arguments
    ///
    /// * `domain` - A reference to the `Domain` to add as a subdomain.
    ///
    /// # Side Effects
    ///
    /// * Inserts a new entry in the domains map with the full subdomain name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use webserver::{WebServer, ServerConfig, Domain};
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let mut server = WebServer::new(config);
    /// let domain = Domain::new("api");
    /// server.add_subdomain_router(&domain);
    /// ```
    pub fn add_subdomain_router(&mut self, domain: &Domain) {
        let mut guard = self.domains.lock().unwrap();
        let domain_str = format!(
            "{}.{}",
            domain.name.to_lowercase(),
            self.default_domain.name
        );
        guard
            .entry(Domain::new(&*domain_str))
            .or_insert_with(DomainRoutes::new);
    }

    /// Adds a file-based route to the specified domain.
    ///
    /// # Arguments
    ///
    /// * `route` - The route pattern to match (e.g., "/about").
    /// * `file_path` - The path to the file whose content will be served.
    /// * `domain` - Optional reference to the domain; if None, uses default domain.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on successful addition.
    /// * `Err(String)` if there's an error (e.g., file doesn't exist).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use webserver::{WebServer, ServerConfig, Domain};
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let mut server = WebServer::new(config);
    /// server.add_route_file("/about", "./static/about.html", None);
    /// ```
    pub fn add_route_file(
        &mut self,
        route: &str,
        file_path: &str,
        domain: Option<&Domain>,
    ) -> &mut Self {
        let domain = domain
            .cloned()
            .unwrap_or_else(|| self.default_domain.clone());

        let content = get_file_content(&PathBuf::from(file_path));

        {
            let mut guard = self.domains.lock().unwrap();
            guard
                .entry(domain.clone())
                .or_insert_with(DomainRoutes::new)
                .routes
                .insert(route.to_string(), content.to_string());
        }
        self
    }

    /// Adds a static file route to the specified domain.
    ///
    /// # Arguments
    ///
    /// * `route` - The route pattern to match (e.g., "/static").
    /// * `folder` - The path to the folder containing static files.
    /// * `domain` - Optional reference to the domain; if None, uses default domain.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on successful addition.
    /// * `Err(String)` if there's an error (e.g., folder doesn't exist).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use webserver::{WebServer, ServerConfig, Domain};
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let mut server = WebServer::new(config);
    /// server.add_static_route("/assets", "./static/assets", None);
    /// ```
    pub fn add_static_route(
        &mut self,
        route: &str,
        folder: &str,
        domain: Option<&Domain>,
    ) -> &mut Self {
        let domain = domain.unwrap_or_else(|| &self.default_domain);
        let folder_path = PathBuf::from(folder);
        if !folder_path.exists() {
            error!("Static route file does not exist");
        }
        {
            let mut guard = self.domains.lock().unwrap();
            if !guard.contains_key(&domain) {
                guard.insert(domain.clone(), DomainRoutes::new());
            }
            match guard.get_mut(&domain) {
                Some(domain_routes) => {
                    domain_routes
                        .static_routes
                        .insert(route.to_string(), folder.to_string());
                }
                None => error!("Domain not found: {}", domain.name),
            }
        }
        self
    }

    /// Adds a custom route handler to the specified domain.
    ///
    /// # Arguments
    ///
    /// * `route` - The route pattern to match (e.g., "/api").
    /// * `f` - A closure or function that takes a `Request` and `Domain` and returns a `Response`.
    /// * `domain` - Optional reference to the domain; if None, uses default domain.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on successful addition.
    /// * `Err(String)` if the domain is not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use webserver::{WebServer, ServerConfig, Domain, Request, Response};
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let server = WebServer::new(config);
    /// server.add_custom_route("/api", |req, domain| {
    ///     // Custom logic here
    ///     Response::new(200, "Hello from API".to_string())
    /// }, None);
    /// ```
    pub fn add_custom_route(
        &mut self,
        route: &str,
        f: impl Fn(Request, &Domain) -> Response + Send + Sync + 'static,
        domain: Option<&Domain>,
    ) -> &mut Self {
        let domain = domain.unwrap_or_else(|| &self.default_domain);
        {
            match self.domains.lock().unwrap().get_mut(&domain) {
                Some(domain_routes) => {
                    domain_routes
                        .custom_routes
                        .insert(route.to_string(), Arc::new(f));
                }
                None => error!("Domain not found: {}", domain.name),
            }
        }
        self
    }

    /// Adds a route to a specific on error
    ///
    /// # Arguments
    ///
    /// * `status_code` - The Code it should react on.
    /// * `file` - the file that will be shown on the code.
    /// * `domain` - Optional reference to the domain; if None, uses default domain.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on successful addition.
    /// * `Err(String)` if the domain is not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sunweb::webserver::responses::ResponseCodes;
    /// use webserver::{WebServer, ServerConfig, Domain, Request, Response};
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let server = WebServer::new(config);
    /// server.add_error_route(ResponseCodes::NotFound, "./resources/errors/404.html", None);
    /// ```
    pub fn add_error_route(
        &mut self,
        status_code: ResponseCodes,
        file: &str,
        domain: Option<&Domain>,
    ) -> &mut Self {
        let domain = domain
            .cloned()
            .unwrap_or_else(|| self.default_domain.clone());

        let content = get_file_content(&PathBuf::from(file));

        {
            let mut guard = self.domains.lock().unwrap();
            guard
                .entry(domain.clone())
                .or_insert_with(DomainRoutes::new)
                .code_routes
                .insert(status_code, content.to_string());
        }
        self
    }

    /// Adds a route to a specific on error
    ///
    /// # Arguments
    ///
    /// * `route` - The path to open it.
    /// * `external_url` - The url to the external service where to proxy too.
    /// * `domain` - Optional reference to the domain; if None, uses default domain.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on successful addition.
    /// * `Err(String)` if the domain is not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sunweb::webserver::responses::ResponseCodes;
    /// use webserver::{WebServer, ServerConfig, Domain, Request, Response};
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let server = WebServer::new(config);
    /// server.add_proxy_route("/", "https://github.com/Sunnickel", None);
    /// ```
    pub fn add_proxy_route(
        &mut self,
        route: &str,
        external_url: &str,
        domain: Option<&Domain>,
    ) -> &mut Self {
        let domain = domain
            .cloned()
            .unwrap_or_else(|| self.default_domain.clone());

        {
            let mut guard = self.domains.lock().unwrap();
            guard
                .entry(domain.clone())
                .or_insert_with(DomainRoutes::new)
                .proxy_route
                .insert(route.to_string(), external_url.to_string());
        }
        self
    }

    /// A logging middleware function that logs request and response details.
    ///
    /// # Arguments
    ///
    /// * `request` - Mutable reference to the incoming `Request`.
    /// * `response` - The `Response` to be sent back.
    ///
    /// # Returns
    ///
    /// The same `Response` passed in, unchanged.
    ///
    /// # Examples
    ///
    /// This function is used internally by the server for logging.
    pub(crate) fn logging(request: &mut Request, response: Response) -> Response {
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

    /// A error page middleware function that allows to override 404 pages and more.
    ///
    /// # Arguments
    ///
    /// * `request` - Mutable reference to the incoming `Request`.
    /// * `response` - The `Response` to be sent back.
    /// * `domain_routes` - Just the domainroutes.
    ///
    /// # Returns
    ///
    /// The same `Response` passed in, unchanged.
    ///
    /// # Examples
    ///
    /// This function is used internally to replace error code pages.
    pub(crate) fn errorpage(
        request: &mut Request,
        response: Response,
        domain_routes: &DomainRoutes,
    ) -> Response {
        let status_code = response.headers.status;

        if let Some(content) = domain_routes.code_routes.get(&status_code.clone()) {
            return Response::new(Arc::new(content.to_string()), Some(status_code), None);
        }

        response
    }
}
