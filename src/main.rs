use crate::webserver::responses::{Response, generate_response};
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
    server.add_subdomain_server(api.clone());

    server.add_route_file("/", "./resources/templates/index.html", Some(api));
    server.add_route_file("/", "./resources/templates/index.html", None);
    server.add_custom_route("/custom", custom_route, None);
    server.add_static_route("/static", "./resources/static", None);
    server.start();
}

fn custom_route(_request: String) -> String {
    let response = Response::new(Arc::new(String::from("<p> Custom Thingie </p>")));
    generate_response(&response)
}
