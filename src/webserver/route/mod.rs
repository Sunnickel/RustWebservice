use crate::webserver::requests::Request;
use crate::webserver::responses::{Response, ResponseCodes};
use crate::webserver::Domain;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RouteType {
    Static,
    File,
    Custom,
    Error,
    Proxy,
}

#[derive(Clone)]
pub enum HTTPMethod {
    GET,
    HEAD,
    OPTIONS,
    TRACE,
    PUT,
    DELETE,
    POST,
    PATCH,
    CONNECT,
}

impl fmt::Display for HTTPMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HTTPMethod::GET => write!(f, "GET"),
            HTTPMethod::POST => write!(f, "POST"),
            HTTPMethod::PUT => write!(f, "PUT"),
            HTTPMethod::DELETE => write!(f, "DELETE"),
            HTTPMethod::PATCH => write!(f, "PATCH"),
            HTTPMethod::OPTIONS => write!(f, "OPTIONS"),
            HTTPMethod::HEAD => write!(f, "HEAD"),
            _ => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Route {
    pub(crate) route: String,
    pub(crate) domain: Domain,
    pub(crate) method: HTTPMethod,
    pub(crate) route_type: RouteType,
    pub(crate) status_code: ResponseCodes,
    pub(crate) external: Option<String>,
    pub(crate) content: Option<Arc<String>>,
    pub(crate) folder: Option<String>,
    pub(crate) f: Option<Arc<dyn Fn(Request, &Domain) -> Response + Send + Sync>>,
}

impl Route {
    pub(crate) fn new_file(
        route: String,
        method: HTTPMethod,
        response_code: ResponseCodes,
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

    pub(crate) fn new_custom(
        route: String,
        method: HTTPMethod,
        response_code: ResponseCodes,
        domain: Domain,
        f: impl Fn(Request, &Domain) -> Response + Send + Sync + 'static,
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

    pub(crate) fn new_static(
        route: String,
        method: HTTPMethod,
        response_code: ResponseCodes,
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

    pub(crate) fn new_error(
        method: HTTPMethod,
        domain: Domain,
        response_code: ResponseCodes,
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

    pub(crate) fn new_proxy(
        route: String,
        method: HTTPMethod,
        domain: Domain,
        response_code: ResponseCodes,
        external: String,
    ) -> Route {
        Self {
            route,
            domain,
            method,
            route_type: RouteType::Error,
            status_code: response_code,
            external: Some(external),
            content: None,
            folder: None,
            f: None,
        }
    }
}
