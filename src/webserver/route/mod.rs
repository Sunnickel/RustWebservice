mod http_method;

use crate::webserver::Domain;
use crate::webserver::requests::HTTPRequest;
use crate::webserver::responses::{HTTPResponse, StatusCode};
pub use crate::webserver::route::http_method::HTTPMethod;
use std::sync::Arc;

/// Represents the type of a route.
///
/// This is used internally by the web server to determine how to handle requests.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RouteType {
    /// Serves files from a static folder.
    Static,
    /// Serves a specific file.
    File,
    /// Uses a custom closure to generate a response.
    Custom,
    /// Represents an error page route (e.g., 404).
    Error,
    /// Forwards the request to an external URL.
    Proxy,
}

/// Represents a route in the web server.
///
/// Contains all the information needed to match requests and generate responses.
#[derive(Clone)]
pub(crate) struct Route {
    /// The path or route string.
    pub(crate) route: String,
    /// The domain this route belongs to.
    pub(crate) domain: Domain,
    /// The HTTP method for this route.
    pub(crate) method: HTTPMethod,
    /// The type of route (Static, File, Custom, Error, Proxy).
    pub(crate) route_type: RouteType,
    /// The HTTP response code for this route.
    pub(crate) status_code: StatusCode,
    /// The external URL for proxy routes.
    pub(crate) external: Option<String>,
    /// Optional content for file or error routes.
    pub(crate) content: Option<Arc<String>>,
    /// Optional folder path for static routes.
    pub(crate) folder: Option<String>,
    /// Optional custom closure for dynamic routes.
    pub(crate) f: Option<Arc<dyn Fn(HTTPRequest, &Domain) -> HTTPResponse + Send + Sync>>,
}

impl Route {
    /// Creates a new file-based route.
    ///
    /// # Arguments
    ///
    /// * `route` - Route path (e.g., "/about.html").
    /// * `method` - HTTP method for this route.
    /// * `response_code` - HTTP status code for responses.
    /// * `domain` - Domain this route belongs to.
    /// * `content` - File content to serve.
    ///
    /// # Returns
    ///
    /// A `Route` configured to serve a single file.
    pub(crate) fn new_file(
        route: String,
        method: HTTPMethod,
        response_code: StatusCode,
        domain: Domain,
        content: Arc<String>,
    ) -> Route {
        Self {
            route,
            domain,
            method,
            route_type: RouteType::File,
            status_code: response_code,
            external: None,
            content: Some(content),
            folder: None,
            f: None,
        }
    }

    /// Creates a new custom route using a closure.
    ///
    /// # Arguments
    ///
    /// * `route` - Route path.
    /// * `method` - HTTP method.
    /// * `response_code` - HTTP status code.
    /// * `domain` - Domain for this route.
    /// * `f` - Closure that generates an `HTTPResponse` from a request.
    ///
    /// # Returns
    ///
    /// A `Route` that calls the custom closure when matched.
    pub(crate) fn new_custom(
        route: String,
        method: HTTPMethod,
        response_code: StatusCode,
        domain: Domain,
        f: impl Fn(HTTPRequest, &Domain) -> HTTPResponse + Send + Sync + 'static,
    ) -> Route {
        Self {
            route,
            domain,
            method,
            route_type: RouteType::Custom,
            status_code: response_code,
            external: None,
            content: None,
            folder: None,
            f: Some(Arc::new(f)),
        }
    }

    /// Creates a new static folder route.
    ///
    /// # Arguments
    ///
    /// * `route` - Route path (prefix for folder).
    /// * `method` - HTTP method.
    /// * `response_code` - HTTP status code.
    /// * `domain` - Domain for this route.
    /// * `folder` - Folder path containing static files.
    ///
    /// # Returns
    ///
    /// A `Route` serving files from the specified folder.
    pub(crate) fn new_static(
        route: String,
        method: HTTPMethod,
        response_code: StatusCode,
        domain: Domain,
        folder: String,
    ) -> Route {
        Self {
            route,
            domain,
            method,
            route_type: RouteType::Static,
            status_code: response_code,
            external: None,
            content: None,
            folder: Some(folder),
            f: None,
        }
    }

    /// Creates a new error page route.
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method for the route.
    /// * `domain` - Domain for this route.
    /// * `response_code` - HTTP status code (e.g., 404).
    /// * `content` - Content to display for the error page.
    ///
    /// # Returns
    ///
    /// A `Route` that represents an error page.
    pub(crate) fn new_error(
        method: HTTPMethod,
        domain: Domain,
        response_code: StatusCode,
        content: Arc<String>,
    ) -> Route {
        Self {
            route: String::new(),
            domain,
            method,
            route_type: RouteType::Error,
            status_code: response_code,
            external: None,
            content: Some(content),
            folder: None,
            f: None,
        }
    }

    /// Creates a new proxy route forwarding requests to an external URL.
    ///
    /// # Arguments
    ///
    /// * `route` - Route path to match.
    /// * `method` - HTTP method.
    /// * `domain` - Domain for this route.
    /// * `response_code` - HTTP status code.
    /// * `external` - External URL to proxy the request to.
    ///
    /// # Returns
    ///
    /// A `Route` that proxies requests to an external server.
    pub(crate) fn new_proxy(
        route: String,
        method: HTTPMethod,
        domain: Domain,
        response_code: StatusCode,
        external: String,
    ) -> Route {
        Self {
            route,
            domain,
            method,
            route_type: RouteType::Proxy,
            status_code: response_code,
            external: Some(external),
            content: None,
            folder: None,
            f: None,
        }
    }
}
