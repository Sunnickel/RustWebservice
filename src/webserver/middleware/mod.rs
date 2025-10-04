use crate::webserver::Domain;
use crate::webserver::requests::Request;
use crate::webserver::responses::Response;

pub enum MiddlewareFn {
    Request(fn(&mut Request)),
    Response(fn(&mut Response)),
    BothResponse(fn(&mut Request, Response) -> Response),
    Both(fn(Request) -> Request, fn(Response) -> Response),
}

pub struct Middleware {
    pub(crate) domain: Domain,
    pub(crate) route: String,
    pub(crate) f: MiddlewareFn,
}

impl Middleware {
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
