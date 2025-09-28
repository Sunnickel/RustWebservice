use crate::webserver::WebServer;

mod webserver;

fn main() {
    let mut server = WebServer::new([0, 0, 0, 0], 80);

    server.add_route("/", "./resources/templates/index.html");

    server.add_static_route("/static", "./resources/static");
    server.start();
}
