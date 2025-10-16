use crate::webserver::requests::HTTPRequest;
use crate::webserver::responses::HTTPResponse;
use crate::webserver::route::Route;
use crate::webserver::Domain;

/// Represents a function that can be used as middleware in the web server.
///
/// This enum defines different types of middleware functions that can be applied
/// to HTTPRequests, HTTPResponses, or both. Each variant corresponds to a specific
/// middleware behavior:
///
/// - `HTTPRequest`: A function that modifies a mutable HTTPRequest.
/// - `HTTPResponse`: A function that modifies a mutable HTTPResponse.
/// - `BothHTTPResponse`: A function that takes a mutable HTTPRequest and an immutable HTTPResponse,
///   and returns a modified HTTPResponse.
/// - `Both`: Two separate functions, one for modifying HTTPRequests and one for HTTPResponses.
pub enum MiddlewareFn {
    HTTPRequest(fn(&mut HTTPRequest)),
    HTTPResponse(fn(&mut HTTPResponse)),
    BothHTTPResponse(fn(&mut HTTPRequest, HTTPResponse) -> HTTPResponse),
    Both(
        fn(HTTPRequest) -> HTTPRequest,
        fn(HTTPResponse) -> HTTPResponse,
    ),
    HTTPResponseBothWithRoutes(fn(&mut HTTPRequest, HTTPResponse, &Vec<Route>) -> HTTPResponse),
}

/// A middleware component that can be applied to HTTP HTTPRequests and HTTPResponses.
///
/// This struct holds the domain, route, and function associated with a middleware.
/// It allows for flexible routing of middleware based on domain and route patterns.
pub struct Middleware {
    /// The domain pattern this middleware applies to.
    ///
    /// If `None`, it defaults to "*" which matches all domains.
    pub(crate) domain: Domain,

    /// The route pattern this middleware applies to.
    ///
    /// If `None`, it defaults to "*" which matches all routes.
    pub(crate) route: String,

    /// The function that implements the middleware behavior.
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
    /// use your_crate::webserver::{Domain, Middleware};
    /// use your_crate::webserver::HTTPRequests::HTTPRequest;
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
    /// use your_crate::webserver::{Domain, Middleware};
    /// use your_crate::webserver::HTTPResponses::HTTPResponse;
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
    /// use your_crate::webserver::{Domain, Middleware};
    /// use your_crate::webserver::HTTPRequests::HTTPRequest;
    /// use your_crate::webserver::HTTPResponses::HTTPResponse;
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
    /// use your_crate::webserver::{Domain, Middleware};
    /// use your_crate::webserver::HTTPRequests::HTTPRequest;
    /// use your_crate::webserver::HTTPResponses::HTTPResponse;
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

    pub fn new_response_both_w_routes(
        domain: Option<Domain>,
        route: Option<String>,
        f: fn(&mut HTTPRequest, HTTPResponse, &Vec<Route>) -> HTTPResponse,
    ) -> Middleware {
        Self {
            domain: domain.unwrap_or_else(|| Domain::new("*")),
            route: route.unwrap_or_else(|| "*".to_string()),
            f: MiddlewareFn::HTTPResponseBothWithRoutes(f),
        }
    }
}
