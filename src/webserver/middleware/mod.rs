//! Middleware support for the web-server.
//!
//! Middleware can intercept and/or mutate requests and responses, either
//! globally or only for specific domain + route patterns.
//!
//! # Example
//!
//! ```
//! use webserver::{Domain, Middleware};
//!
//! // Add a request logger for every route under api.example.com
//! let logger = Middleware::new_request(
//!     Some(Domain::new("api.example.com")),
//!     None,                       // any route
//!     |req| eprintln!("→ {} {}", req.method, req.path),
//! );
//! ```
use crate::webserver::Domain;
use crate::webserver::requests::HTTPRequest;
use crate::webserver::responses::HTTPResponse;
use crate::webserver::route::Route;

/// Signature bundle for every supported middleware flavour.
///
/// Variants are deliberately *not* generic so the rest of the server can
/// pattern-match over them without type gymnastics.
pub enum MiddlewareFn {
    /// `fn(&mut HTTPRequest)` – mutate the incoming request.
    HTTPRequest(fn(&mut HTTPRequest)),

    /// `fn(&mut HTTPResponse)` – mutate the outgoing response.
    HTTPResponse(fn(&mut HTTPResponse)),

    /// `fn(&mut Request, Response) -> Response` – decide which response to
    /// send, optionally mutating the request on the way.
    BothHTTPResponse(fn(&mut HTTPRequest, HTTPResponse) -> HTTPResponse),

    /// A pair of *pure* functions: `(Request -> Request, Response -> Response)`.
    Both(
        fn(HTTPRequest) -> HTTPRequest,
        fn(HTTPResponse) -> HTTPResponse,
    ),

    /// Like `BothHTTPResponse` but the current route table is also provided.
    HTTPResponseBothWithRoutes(fn(&mut HTTPRequest, HTTPResponse, &[Route]) -> HTTPResponse),
}

/// A middleware rule: domain pattern + route pattern + one of the functions
/// above.
pub struct Middleware {
    /// Domain that must match (or `*` for any).
    pub(crate) domain: Domain,
    /// Route prefix that must match (or `*` for any).
    pub(crate) route: String,
    /// Function(s) to execute.
    pub(crate) f: MiddlewareFn,
}

impl Middleware {
    /// Creates a new middleware that modifies HTTPRequests.
    ///
    /// # Description
    ///
    /// This constructor creates a middleware that will be applied to incoming
    /// HTTP HTTPRequests. It allows specifying a domain and route pattern for when
    /// the middleware should be executed.
    ///
    /// # Arguments
    ///
    /// * `domain`: An optional domain pattern. If `None`, defaults to "*".
    /// * `route`: An optional route pattern. If `None`, defaults to "*".
    /// * `f`: A function that takes a mutable reference to a HTTPRequest and modifies it.
    ///
    /// # Returns
    ///
    /// A new Middleware instance with the specified parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use sunweb::webserver::{Domain, Middleware};
    /// use sunweb::webserver::HTTPRequests::HTTPRequest;
    ///
    /// fn modify_HTTPRequest(req: &mut HTTPRequest) {
    ///     // Modify HTTPRequest here
    /// }
    ///
    /// let middleware = Middleware::new_HTTPRequest(
    ///     Some(Domain::new("example.com")),
    ///     Some("/api".to_string()),
    ///     modify_HTTPRequest,
    /// );
    /// ```
    pub fn new_request(
        domain: Option<Domain>,
        route: Option<String>,
        f: fn(&mut HTTPRequest),
    ) -> Middleware {
        Self {
            domain: domain.unwrap_or_else(|| Domain::new("*")),
            route: route.unwrap_or_else(|| "*".to_string()),
            f: MiddlewareFn::HTTPRequest(f),
        }
    }

    /// Creates a new middleware that modifies HTTPResponses.
    ///
    /// # Description
    ///
    /// This constructor creates a middleware that will be applied to outgoing
    /// HTTP HTTPResponses. It allows specifying a domain and route pattern for when
    /// the middleware should be executed.
    ///
    /// # Arguments
    ///
    /// * `domain`: An optional domain pattern. If `None`, defaults to "*".
    /// * `route`: An optional route pattern. If `None`, defaults to "*".
    /// * `f`: A function that takes a mutable reference to a HTTPResponse and modifies it.
    ///
    /// # Returns
    ///
    /// A new Middleware instance with the specified parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use sunweb::webserver::{Domain, Middleware};
    /// use sunweb::webserver::HTTPResponses::HTTPResponse;
    ///
    /// fn modify_HTTPResponse(res: &mut HTTPResponse) {
    ///     // Modify HTTPResponse here
    /// }
    ///
    /// let middleware = Middleware::new_HTTPResponse(
    ///     Some(Domain::new("example.com")),
    ///     Some("/api".to_string()),
    ///     modify_HTTPResponse,
    /// );
    /// ```
    pub fn new_response(
        domain: Option<Domain>,
        route: Option<String>,
        f: fn(&mut HTTPResponse),
    ) -> Middleware {
        Self {
            domain: domain.unwrap_or_else(|| Domain::new("*")),
            route: route.unwrap_or_else(|| "*".to_string()),
            f: MiddlewareFn::HTTPResponse(f),
        }
    }

    /// Creates a new middleware that modifies both HTTPRequests and HTTPResponses.
    ///
    /// # Description
    ///
    /// This constructor creates a middleware that will modify both incoming
    /// HTTP HTTPRequests and outgoing HTTPResponses. It allows specifying a domain
    /// and route pattern for when the middleware should be executed.
    ///
    /// # Arguments
    ///
    /// * `domain`: An optional domain pattern. If `None`, defaults to "*".
    /// * `route`: An optional route pattern. If `None`, defaults to "*".
    /// * `f_req`: A function that takes a HTTPRequest and returns a modified HTTPRequest.
    /// * `f_res`: A function that takes a HTTPResponse and returns a modified HTTPResponse.
    ///
    /// # Returns
    ///
    /// A new Middleware instance with the specified parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use sunweb::webserver::{Domain, Middleware};
    /// use sunweb::webserver::HTTPRequests::HTTPRequest;
    /// use sunweb::webserver::HTTPResponses::HTTPResponse;
    ///
    /// fn modify_HTTPRequest(req: HTTPRequest) -> HTTPRequest {
    ///     // Modify HTTPRequest here
    ///     req
    /// }
    ///
    /// fn modify_HTTPResponse(res: HTTPResponse) -> HTTPResponse {
    ///     // Modify HTTPResponse here
    ///     res
    /// }
    ///
    /// let middleware = Middleware::new_both(
    ///     Some(Domain::new("example.com")),
    ///     Some("/api".to_string()),
    ///     modify_HTTPRequest,
    ///     modify_HTTPResponse,
    /// );
    /// ```
    pub fn new_both(
        domain: Option<Domain>,
        route: Option<String>,
        f_req: fn(HTTPRequest) -> HTTPRequest,
        f_res: fn(HTTPResponse) -> HTTPResponse,
    ) -> Middleware {
        Self {
            domain: domain.unwrap_or_else(|| Domain::new("*")),
            route: route.unwrap_or_else(|| "*".to_string()),
            f: MiddlewareFn::Both(f_req, f_res),
        }
    }

    /// Creates a new middleware that modifies HTTPRequests and returns modified HTTPResponses.
    ///
    /// # Description
    ///
    /// This constructor creates a middleware that takes a mutable HTTPRequest and an
    /// immutable HTTPResponse, and returns a modified HTTPResponse. It allows specifying
    /// a domain and route pattern for when the middleware should be executed.
    ///
    /// # Arguments
    ///
    /// * `domain`: An optional domain pattern. If `None`, defaults to "*".
    /// * `route`: An optional route pattern. If `None`, defaults to "*".
    /// * `f`: A function that takes a mutable reference to a HTTPRequest and an immutable HTTPResponse,
    ///   and returns a modified HTTPResponse.
    ///
    /// # Returns
    ///
    /// A new Middleware instance with the specified parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use sunweb::webserver::{Domain, Middleware};
    /// use sunweb::webserver::HTTPRequests::HTTPRequest;
    /// use sunweb::webserver::HTTPResponses::HTTPResponse;
    ///
    /// fn modify_HTTPRequest_and_HTTPResponse(req: &mut HTTPRequest, res: HTTPResponse) -> HTTPResponse {
    ///     // Modify HTTPRequest and HTTPResponse here
    ///     res
    /// }
    ///
    /// let middleware = Middleware::new_HTTPResponse_both(
    ///     Some(Domain::new("example.com")),
    ///     Some("/api".to_string()),
    ///     modify_HTTPRequest_and_HTTPResponse,
    /// );
    /// ```
    pub fn new_response_both(
        domain: Option<Domain>,
        route: Option<String>,
        f: fn(&mut HTTPRequest, HTTPResponse) -> HTTPResponse,
    ) -> Middleware {
        Self {
            domain: domain.unwrap_or_else(|| Domain::new("*")),
            route: route.unwrap_or_else(|| "*".to_string()),
            f: MiddlewareFn::BothHTTPResponse(f),
        }
    }

    /// Like [`new_response_both`](Self::new_response_both) but the current
    /// route table is also provided (useful for dynamic routing or logging).
    pub fn new_response_both_w_routes(
        domain: Option<Domain>,
        route: Option<String>,
        f: fn(&mut HTTPRequest, HTTPResponse, &[Route]) -> HTTPResponse,
    ) -> Middleware {
        Self {
            domain: domain.unwrap_or_else(|| Domain::new("*")),
            route: route.unwrap_or_else(|| "*".to_string()),
            f: MiddlewareFn::HTTPResponseBothWithRoutes(f),
        }
    }
}
