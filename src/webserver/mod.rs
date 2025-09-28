mod client_handling;
mod files;
mod responses;

use crate::webserver::client_handling::Client;
use crate::webserver::files::get_file_content;
use crate::webserver::responses::generate_response;
use std::collections::HashMap;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

pub struct WebServer {
    pub(crate) host: [u8; 4],
    pub port: u16,
    routes: HashMap<String, Arc<dyn Fn() -> String + Send + Sync + 'static>>,
    static_routes: HashMap<String, String>,
}

impl WebServer {
    pub fn new(host: [u8; 4], port: u16) -> WebServer {
        WebServer {
            host,
            port,
            routes: HashMap::new(),
            static_routes: HashMap::new(),
        }
    }

    pub fn start(&self) {
        let bind_addr = format!(
            "{}.{}.{}.{}:{}",
            self.host[0], self.host[1], self.host[2], self.host[3], self.port
        );

        let listener = TcpListener::bind(&bind_addr).unwrap();
        println!("Server running on http://{bind_addr}/");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let routes = Arc::new(self.routes.clone());
                    let static_routes = Arc::new(self.static_routes.clone());

                    thread::spawn(move || {
                        let mut client: Client = Client::new(stream, &routes, &static_routes);
                        client.handle();
                    });
                }
                Err(e) => eprintln!("Connection failed: {e}"),
            }
        }
    }

    pub fn add_route(&mut self, route: &str, file_path: &str) {
        let content = get_file_content(&*PathBuf::from(&file_path));

        self.routes.insert(
            route.to_string(),
            Arc::new(move || generate_response(&**content)),
        );
    }

    pub fn add_static_route(&mut self, route: &str, folder: &str) {
        let folder_path = PathBuf::from(folder);

        if folder_path.exists() {
            self.static_routes
                .insert(route.to_string(), folder.to_string());
        } else {
            panic!("Folder doesn't exist");
        }
    }
}
