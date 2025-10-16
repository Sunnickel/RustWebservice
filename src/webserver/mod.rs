//! A web server implementation with domain routing capabilities.
//!
//! This module provides the core components for building a web server,
//! including domain handling, routing, middleware support, and request/response processing.

mod client_handling;
pub(crate) mod files;
pub mod http_packet;
pub(crate) mod logger;
pub(crate) mod middleware;
mod proxy;
pub mod requests;
pub mod responses;
pub mod route;
pub(crate) mod server_config;

use crate::webserver::client_handling::Client;
use crate::webserver::files::get_file_content;
use crate::webserver::middleware::Middleware;
use crate::webserver::route::{HTTPMethod, Route, RouteType};
pub use crate::webserver::server_config::ServerConfig;

use crate::webserver::http_packet::header::connection::ConnectionType;
use crate::webserver::logger::Logger;
use crate::webserver::requests::HTTPRequest;
use crate::webserver::responses::{HTTPResponse, StatusCode};
use chrono::Utc;
use log::{error, info, Level, LevelFilter, Log, Metadata, Record};
use std::collections::HashMap;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

/// ANSI color code for red text.
const RED: &str = "\x1b[31m";

/// ANSI color code for yellow text.
const YELLOW: &str = "\x1b[33m";

/// ANSI color code for blue text.
const BLUE: &str = "\x1b[34m";

/// ANSI color code for green text.
const GREEN: &str = "\x1b[32m";

/// ANSI color code for dimmed text.
const DIM: &str = "\x1b[2m";

/// ANSI color code to reset text formatting.
const RESET: &str = "\x1b[0m";
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

/// The main web server structure.
pub struct WebServer {
    /// Server configuration including IP, port, and TLS settings.
    pub(crate) config: ServerConfig,
    /// Map of domains to their respective routing configurations.
    pub(crate) domains: Arc<Mutex<HashMap<Domain, Arc<Mutex<Vec<Route>>>>>>,
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
        domains.insert(default_domain.clone(), Arc::new(Mutex::new(Vec::new())));
        let mut middlewares = Vec::new();

        let logging_start_middleware =
            Middleware::new_request(None, None, Logger::log_request_start);
        let logging_end_middleware = Middleware::new_response(None, None, Logger::log_request_end);
        let error_page_middleware =
            Middleware::new_response_both_w_routes(None, None, Self::errorpage);

        middlewares.push(logging_start_middleware);
        middlewares.push(logging_end_middleware);
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
                    let default_domain = self.default_domain.clone();
                    let tls_config = self.config.tls_config.clone();

                    thread::spawn(move || {
                        let mut client =
                            Client::new(stream, domains, default_domain, middleware, tls_config);

                        let mut i = 0;
                        loop {
                            match client.handle(i) {
                                Some(connection_type) => match connection_type {
                                    ConnectionType::KeepAlive => {
                                        i += 1;
                                        continue;
                                    }
                                    _ => {
                                        error!("Connection closed: {connection_type}");
                                        break;
                                    }
                                },
                                None => break,
                            };
                        }
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
            .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));
    }

    /// Adds a file-based route to the specified domain.
    ///
    /// # Arguments
    ///
    /// * `route` - The route pattern to match (e.g., "/about").
    /// * `method` - The HTTP method for this route.
    /// * `file_path` - The path to the file whose content will be served.
    /// * `response_codes` - The response code for successful responses.
    /// * `domain` - Optional reference to the domain; if None, uses default domain.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use webserver::{WebServer, ServerConfig, Domain};
    /// use webserver::route::HTTPMethod;
    /// use webserver::responses::StatusCode;
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let mut server = WebServer::new(config);
    /// server.add_route_file("/about", HTTPMethod::GET, "./static/about.html", StatusCode::Ok, None);
    /// ```
    pub fn add_route_file(
        &mut self,
        route: &str,
        method: HTTPMethod,
        file_path: &str,
        response_codes: StatusCode,
        domain: Option<&Domain>,
    ) -> &mut Self {
        let domain = domain
            .cloned()
            .unwrap_or_else(|| self.default_domain.clone());

        let content = get_file_content(&PathBuf::from(file_path));

        {
            let mut guard = self.domains.lock().unwrap();
            let domain_routes = guard
                .entry(domain.clone())
                .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));

            let mut routes = domain_routes.lock().unwrap();
            routes.push(Route::new_file(
                route.to_string(),
                method,
                response_codes,
                domain,
                content,
            ));
        }

        self
    }

    /// Adds a static file route to the specified domain.
    ///
    /// # Arguments
    ///
    /// * `route` - The route pattern to match (e.g., "/static").
    /// * `method` - The HTTP method for this route.
    /// * `folder` - The path to the folder containing static files.
    /// * `response_codes` - The response code for successful responses.
    /// * `domain` - Optional reference to the domain; if None, uses default domain.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use webserver::{WebServer, ServerConfig, Domain};
    /// use webserver::route::HTTPMethod;
    /// use webserver::responses::StatusCode;
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let mut server = WebServer::new(config);
    /// server.add_static_route("/assets", HTTPMethod::GET, "./static/assets", StatusCode::Ok, None);
    /// ```
    pub fn add_static_route(
        &mut self,
        route: &str,
        method: HTTPMethod,
        folder: &str,
        response_codes: StatusCode,
        domain: Option<&Domain>,
    ) -> &mut Self {
        let domain = domain
            .cloned()
            .unwrap_or_else(|| self.default_domain.clone());

        let folder_path = PathBuf::from(folder);
        if !folder_path.exists() {
            error!("Static route file does not exist");
        }

        {
            let mut guard = self.domains.lock().unwrap();
            let domain_routes = guard
                .entry(domain.clone())
                .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));

            let mut routes = domain_routes.lock().unwrap();
            routes.push(Route::new_static(
                route.to_string(),
                method,
                response_codes,
                domain,
                String::from(folder),
            ));
        }
        self
    }

    /// Adds a custom route handler to the specified domain.
    ///
    /// # Arguments
    ///
    /// * `route` - The route pattern to match (e.g., "/api").
    /// * `method` - The HTTP method for this route.
    /// * `f` - A closure or function that takes a `HTTPRequest` and `Domain` and returns a `HTTPResponse`.
    /// * `response_codes` - The response code for successful responses.
    /// * `domain` - Optional reference to the domain; if None, uses default domain.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use webserver::{WebServer, ServerConfig, Domain, HTTPRequest, HTTPResponse};
    /// use webserver::route::HTTPMethod;
    /// use webserver::responses::StatusCode;
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let mut server = WebServer::new(config);
    /// server.add_custom_route("/api", HTTPMethod::GET, |req, domain| {
    ///     // Custom logic here
    ///     HTTPResponse::new(200, "Hello from API".to_string())
    /// }, StatusCode::Ok, None);
    /// ```
    pub fn add_custom_route(
        &mut self,
        route: &str,
        method: HTTPMethod,
        f: impl Fn(HTTPRequest, &Domain) -> HTTPResponse + Send + Sync + 'static,
        response_codes: StatusCode,
        domain: Option<&Domain>,
    ) -> &mut Self {
        let domain = domain
            .cloned()
            .unwrap_or_else(|| self.default_domain.clone());
        {
            let mut guard = self.domains.lock().unwrap();
            let domain_routes = guard
                .entry(domain.clone())
                .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));

            let mut routes = domain_routes.lock().unwrap();
            routes.push(Route::new_custom(
                route.to_string(),
                method,
                response_codes,
                domain,
                f,
            ));
        }
        self
    }

    /// Adds a route to a specific on error
    ///
    /// # Arguments
    ///
    /// * `status_code` - The Code it should react on.
    /// * `file` - the file that will be shown on the code.
    /// * `response_codes` - The response code configuration.
    /// * `domain` - Optional reference to the domain; if None, uses default domain.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sunweb::webserver::responses::StatusCode;
    /// use webserver::{WebServer, ServerConfig, Domain, HTTPRequest, HTTPResponse};
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let mut server = WebServer::new(config);
    /// server.add_error_route(StatusCode::NotFound, "./resources/errors/404.html", StatusCode::NotFound, None);
    /// ```
    pub fn add_error_route(
        &mut self,
        status_code: StatusCode,
        file: &str,
        response_codes: StatusCode,
        domain: Option<&Domain>,
    ) -> &mut Self {
        let domain = domain
            .cloned()
            .unwrap_or_else(|| self.default_domain.clone());

        let content = get_file_content(&PathBuf::from(file));

        {
            let mut guard = self.domains.lock().unwrap();
            let domain_routes = guard
                .entry(domain.clone())
                .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));

            let mut routes = domain_routes.lock().unwrap();
            routes.push(Route::new_error(
                HTTPMethod::GET,
                domain,
                response_codes,
                content,
            ));
        }
        self
    }

    /// Adds a proxy route to forward requests to an external service
    ///
    /// # Arguments
    ///
    /// * `route` - The path to open it.
    /// * `external_url` - The url to the external service where to proxy too.
    /// * `response_codes` - The response code configuration.
    /// * `domain` - Optional reference to the domain; if None, uses default domain.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sunweb::webserver::responses::StatusCode;
    /// use webserver::{WebServer, ServerConfig, Domain, HTTPRequest, HTTPResponse};
    /// let config = ServerConfig::new("127.0.0.1", 8080, "example.com");
    /// let mut server = WebServer::new(config);
    /// server.add_proxy_route("/", "https://github.com/Sunnickel", StatusCode::Ok, None);
    /// ```
    pub fn add_proxy_route(
        &mut self,
        route: &str,
        external: &str,
        response_codes: StatusCode,
        domain: Option<&Domain>,
    ) -> &mut Self {
        let domain = domain
            .cloned()
            .unwrap_or_else(|| self.default_domain.clone());

        {
            let mut guard = self.domains.lock().unwrap();
            let domain_routes = guard
                .entry(domain.clone())
                .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));

            let mut routes = domain_routes.lock().unwrap();
            routes.push(Route::new_proxy(
                route.to_string(),
                HTTPMethod::GET,
                domain,
                response_codes,
                external.to_string(),
            ));
        }
        self
    }

    /// A error page middleware function that allows to override 404 pages and more.
    ///
    /// # Arguments
    ///
    /// * `request` - Mutable reference to the incoming `HTTPRequest`.
    /// * `response` - The `HTTPResponse` to be sent back.
    /// * `routes` - The routes vector for the current domain.
    ///
    /// # Returns
    ///
    /// The same `HTTPResponse` passed in, unchanged or a custom error page.
    ///
    /// # Examples
    ///
    /// This function is used internally to replace error code pages.
    pub(crate) fn errorpage(
        _request: &mut HTTPRequest,
        response: HTTPResponse,
        routes: &Vec<Route>,
    ) -> HTTPResponse {
        let status_code = response.status_code;

        if let Some(route) = routes
            .iter()
            .find(|x| x.route_type == RouteType::Error && x.status_code == status_code)
        {
            if let Some(content) = &route.content {
                let mut response = HTTPResponse::new(status_code);
                response.set_body_string(content.to_string());
            }
        }

        response
    }
}
