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

    let mut config = ServerConfig::new([0, 0, 0, 0], 443);

    config
        .add_cert(
            get_file_content("./certificates/key.pem".as_ref())
                .parse()
                .unwrap(),
            get_file_content("./certificates/cert.pem".as_ref())
                .parse()
                .unwrap(),
        )
        .expect("Couldnt Read Certificates!");

    let mut server = WebServer::new(config);

    let api = Domain::new("api");
    server.add_subdomain_router(api.clone());

    server
        .add_route_file("/", "./resources/templates/index.html", Some(api.clone()))
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
        .add_static_route("/static", "./resources/static", Some(api.clone()))
        .unwrap();
    server.start();
}

fn custom_route(_request: Request) -> Response {
    let response = Response::new(Arc::new(String::from("<p> Custom Thing </p>")), None, None);
    response
}
