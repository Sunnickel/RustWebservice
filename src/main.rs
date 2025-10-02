use crate::webserver::requests::Request;
use crate::webserver::responses::Response;
use crate::webserver::{Domain, WebServer};
use log::LevelFilter;
use std::sync::Arc;

mod logger;
mod webserver;

pub static WEB_LOGGER: logger::Logger = logger::Logger;

fn main() {
    log::set_logger(&WEB_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Trace);

    let mut server = WebServer::new([0, 0, 0, 0], 80);

    let api = Domain::new("api");
    server.add_subdomain_router(api.clone());

    server.add_route_file("/", "./resources/templates/index.html", Some(api.clone()));
    server.add_route_file("/", "./resources/templates/index.html", None);
    server.add_custom_route("/custom", custom_route, None);
    server.add_static_route("/static", "./resources/static", None);
    server.add_static_route("/static", "./resources/static", Some(api.clone()));
    server.start();
}

fn custom_route(_request: Request) -> Response {
    let response = Response::new(Arc::new(String::from("<p> Custom Thing </p>")), None, None);
    response
}
