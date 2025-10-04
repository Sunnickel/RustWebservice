use crate::webserver::cookie::Cookie;
use crate::webserver::cookie::SameSite::Strict;
use crate::webserver::files::get_file_content;
use crate::webserver::requests::Request;
use crate::webserver::responses::Response;
use crate::webserver::server_config::ServerConfig;
use crate::webserver::{Domain, WebServer};
use log::LevelFilter;
use std::sync::Arc;

mod logger;
mod webserver;

pub static WEB_LOGGER: logger::Logger = logger::Logger;

fn main() {
    log::set_logger(&WEB_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    let mut config = ServerConfig::new([0, 0, 0, 0], 443)
        .add_cert(
            get_file_content("./certificates/key.pem".as_ref())
                .parse()
                .unwrap(),
            get_file_content("./certificates/cert.pem".as_ref())
                .parse()
                .unwrap(),
        )
        .expect("Couldnt Read Certificates!")
        .set_base_domain("localhost".to_string());

    let mut server = WebServer::new(config);

    let api = Domain::from("api");
    server.add_subdomain_router(&api);

    server
        .add_route_file("/", "./resources/templates/index.html", Some(&api))
        .unwrap();
    server
        .add_route_file("/", "./resources/templates/index.html", None)
        .unwrap();
    server
        .add_custom_route("/custom", custom_route, None)
        .unwrap();
    server
        .add_static_route("/static", "./resources/static", None)
        .unwrap();
    server
        .add_static_route("/static", "./resources/static", Some(&api))
        .unwrap();
    server.start();
}

fn custom_route(_request: Request, _domain: &Domain) -> Response {
    let mut response = Response::new(Arc::new(String::from("<p> Custom Thing </p>")), None, None);
    let new_cookie: Cookie = Cookie::new("test", "value1", &_domain)
        .secure()
        .http_only()
        .path("/custom")
        .same_site(Strict);

    if _request.get_cookies().is_empty() {
        response.add_cookie(new_cookie);
    } else if _request.get_cookie("test").is_some() {
        response.expire_cookie(new_cookie);
    } else {
        response.add_cookie(new_cookie);
    }

    response
}
