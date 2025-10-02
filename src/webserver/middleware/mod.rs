use crate::webserver::requests::Request;
use crate::webserver::responses::Response;
use crate::webserver::Domain;

pub enum MiddlewareFn {
    Request(fn(&mut Request)),
    Response(fn(&mut Response)),
    Both(fn(&mut Request) -> Request, fn(&mut Response) -> Response),
}

pub struct Middleware {
    pub domain: Option<Domain>,
    pub route: Option<String>,
    pub f: MiddlewareFn,
}

impl Middleware {
    pub fn new_request(
        domain: Option<Domain>,
        route: Option<String>,
        f: fn(&mut Request),
    ) -> Middleware {
        Middleware {
            domain: Some(domain.unwrap_or_else(|| Domain::from("*"))),
            route: Some(route.unwrap_or_else(|| "*".to_string())),
            f: MiddlewareFn::Request(f),
        }
    }

    pub fn new_response(
        domain: Option<Domain>,
        route: Option<String>,
        f: fn(&mut Response),
    ) -> Middleware {
        Middleware {
            domain: Some(domain.unwrap_or_else(|| Domain::from("*"))),
            route: Some(route.unwrap_or_else(|| "*".to_string())),
            f: MiddlewareFn::Response(f),
        }
    }

    pub fn new_both(
        domain: Option<Domain>,
        route: Option<String>,
        f_req: fn(&mut Request) -> Request,
        f_res: fn(&mut Response) -> Response,
    ) -> Middleware {
        Middleware {
            domain: Some(domain.unwrap_or_else(|| Domain::from("*"))),
            route: Some(route.unwrap_or_else(|| "*".to_string())),
            f: MiddlewareFn::Both(f_req, f_res),
        }
    }
}
