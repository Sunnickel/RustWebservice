mod http_method;

use crate::webserver::requests::HTTPRequest;
use crate::webserver::responses::{HTTPResponse, StatusCode};
pub use crate::webserver::route::http_method::HTTPMethod;
use crate::webserver::Domain;
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
pub(crate) struct Route {
    pub(crate) route: String,
    pub(crate) domain: Domain,
    pub(crate) method: HTTPMethod,
    pub(crate) route_type: RouteType,
    pub(crate) status_code: StatusCode,
    pub(crate) external: Option<String>,
    pub(crate) content: Option<Arc<String>>,
    pub(crate) folder: Option<String>,
    pub(crate) f: Option<Arc<dyn Fn(HTTPRequest, &Domain) -> HTTPResponse + Send + Sync>>,
}

impl Route {
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
            route_type: RouteType::Error,
            status_code: response_code,
            external: Some(external),
            content: None,
            folder: None,
            f: None,
        }
    }
}
