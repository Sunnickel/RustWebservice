use crate::webserver::Domain;
use crate::webserver::requests::Request;
use crate::webserver::responses::Response;

/// Represents a function that can be used as middleware in the web server.
///
/// This enum defines different types of middleware functions that can be applied
/// to requests, responses, or both. Each variant corresponds to a specific
/// middleware behavior:
///
/// - `Request`: A function that modifies a mutable request.
/// - `Response`: A function that modifies a mutable response.
/// - `BothResponse`: A function that takes a mutable request and an immutable response,
///   and returns a modified response.
/// - `Both`: Two separate functions, one for modifying requests and one for responses.
pub enum MiddlewareFn {
    Request(fn(&mut Request)),
    Response(fn(&mut Response)),
    BothResponse(fn(&mut Request, Response) -> Response),
    Both(fn(Request) -> Request, fn(Response) -> Response),
}

/// A middleware component that can be applied to HTTP requests and responses.
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
    /// Creates a new middleware that modifies requests.
    ///
    /// # Description
    ///
    /// This constructor creates a middleware that will be applied to incoming
    /// HTTP requests. It allows specifying a domain and route pattern for when
    /// the middleware should be executed.
    ///
    /// # Arguments
    ///
    /// * `domain`: An optional domain pattern. If `None`, defaults to "*".
    /// * `route`: An optional route pattern. If `None`, defaults to "*".
    /// * `f`: A function that takes a mutable reference to a Request and modifies it.
    ///
    /// # Returns
    ///
    /// A new Middleware instance with the specified parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::webserver::{Domain, Middleware};
    /// use your_crate::webserver::requests::Request;
    ///
    /// fn modify_request(req: &mut Request) {
    ///     // Modify request here
    /// }
    ///
    /// let middleware = Middleware::new_request(
    ///     Some(Domain::new("example.com")),
    ///     Some("/api".to_string()),
    ///     modify_request,
    /// );
    /// ```
    pub fn new_request(
        domain: Option<Domain>,
        route: Option<String>,
        f: fn(&mut Request),
    ) -> Middleware {
        Self {
            domain: domain.unwrap_or_else(|| Domain::new("*")),
            route: route.unwrap_or_else(|| "*".to_string()),
            f: MiddlewareFn::Request(f),
        }
    }

    /// Creates a new middleware that modifies responses.
    ///
    /// # Description
    ///
    /// This constructor creates a middleware that will be applied to outgoing
    /// HTTP responses. It allows specifying a domain and route pattern for when
    /// the middleware should be executed.
    ///
    /// # Arguments
    ///
    /// * `domain`: An optional domain pattern. If `None`, defaults to "*".
    /// * `route`: An optional route pattern. If `None`, defaults to "*".
    /// * `f`: A function that takes a mutable reference to a Response and modifies it.
    ///
    /// # Returns
    ///
    /// A new Middleware instance with the specified parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::webserver::{Domain, Middleware};
    /// use your_crate::webserver::responses::Response;
    ///
    /// fn modify_response(res: &mut Response) {
    ///     // Modify response here
    /// }
    ///
    /// let middleware = Middleware::new_response(
    ///     Some(Domain::new("example.com")),
    ///     Some("/api".to_string()),
    ///     modify_response,
    /// );
    /// ```
    pub fn new_response(
        domain: Option<Domain>,
        route: Option<String>,
        f: fn(&mut Response),
    ) -> Middleware {
        Self {
            domain: domain.unwrap_or_else(|| Domain::new("*")),
            route: route.unwrap_or_else(|| "*".to_string()),
            f: MiddlewareFn::Response(f),
        }
    }

    /// Creates a new middleware that modifies both requests and responses.
    ///
    /// # Description
    ///
    /// This constructor creates a middleware that will modify both incoming
    /// HTTP requests and outgoing responses. It allows specifying a domain
    /// and route pattern for when the middleware should be executed.
    ///
    /// # Arguments
    ///
    /// * `domain`: An optional domain pattern. If `None`, defaults to "*".
    /// * `route`: An optional route pattern. If `None`, defaults to "*".
    /// * `f_req`: A function that takes a Request and returns a modified Request.
    /// * `f_res`: A function that takes a Response and returns a modified Response.
    ///
    /// # Returns
    ///
    /// A new Middleware instance with the specified parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::webserver::{Domain, Middleware};
    /// use your_crate::webserver::requests::Request;
    /// use your_crate::webserver::responses::Response;
    ///
    /// fn modify_request(req: Request) -> Request {
    ///     // Modify request here
    ///     req
    /// }
    ///
    /// fn modify_response(res: Response) -> Response {
    ///     // Modify response here
    ///     res
    /// }
    ///
    /// let middleware = Middleware::new_both(
    ///     Some(Domain::new("example.com")),
    ///     Some("/api".to_string()),
    ///     modify_request,
    ///     modify_response,
    /// );
    /// ```
    pub fn new_both(
        domain: Option<Domain>,
        route: Option<String>,
        f_req: fn(Request) -> Request,
        f_res: fn(Response) -> Response,
    ) -> Middleware {
        Self {
            domain: domain.unwrap_or_else(|| Domain::new("*")),
            route: route.unwrap_or_else(|| "*".to_string()),
            f: MiddlewareFn::Both(f_req, f_res),
        }
    }

    /// Creates a new middleware that modifies requests and returns modified responses.
    ///
    /// # Description
    ///
    /// This constructor creates a middleware that takes a mutable request and an
    /// immutable response, and returns a modified response. It allows specifying
    /// a domain and route pattern for when the middleware should be executed.
    ///
    /// # Arguments
    ///
    /// * `domain`: An optional domain pattern. If `None`, defaults to "*".
    /// * `route`: An optional route pattern. If `None`, defaults to "*".
    /// * `f`: A function that takes a mutable reference to a Request and an immutable Response,
    ///   and returns a modified Response.
    ///
    /// # Returns
    ///
    /// A new Middleware instance with the specified parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate::webserver::{Domain, Middleware};
    /// use your_crate::webserver::requests::Request;
    /// use your_crate::webserver::responses::Response;
    ///
    /// fn modify_request_and_response(req: &mut Request, res: Response) -> Response {
    ///     // Modify request and response here
    ///     res
    /// }
    ///
    /// let middleware = Middleware::new_response_both(
    ///     Some(Domain::new("example.com")),
    ///     Some("/api".to_string()),
    ///     modify_request_and_response,
    /// );
    /// ```
    pub fn new_response_both(
        domain: Option<Domain>,
        route: Option<String>,
        f: fn(&mut Request, Response) -> Response,
    ) -> Middleware {
        Self {
            domain: domain.unwrap_or_else(|| Domain::new("*")),
            route: route.unwrap_or_else(|| "*".to_string()),
            f: MiddlewareFn::BothResponse(f),
        }
    }
}
