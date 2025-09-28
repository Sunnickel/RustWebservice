use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Response {
    headers: HashMap<String, String>,
    content: Arc<String>,
}

impl Response {
    pub fn new(content: Arc<String>) -> Response {
        let mut headers = HashMap::new();
        headers.insert(String::from("Date"), Utc::now().to_rfc2822());
        headers.insert(String::from("Server"), String::from("RustWebServer/1.0"));
        headers.insert(String::from("Content-Type"), String::from("text/html"));
        headers.insert(String::from("Content-Length"), content.len().to_string());
        headers.insert(String::from("Connection"), String::from("close"));

        Self { headers, content }
    }

    pub fn header_to_string(&self) -> String {
        let mut output: String = String::from("");

        for (key, value) in &self.headers {
            output.push_str(&format!("{}: {}\r\n", key, value));
        }
        output.push_str("\r\n");
        output
    }
}

pub fn generate_response(response: &Response) -> String {
    let mut output: String = String::from("HTTP/1.1 200 OK\r\n");

    output.push_str(&*response.header_to_string());
    output.push_str(&*response.content);
    output
}

pub fn generate_static_response(response: &mut Response, content_type: &str) -> String {
    response
        .headers
        .insert(String::from("Content-Type"), content_type.to_string());
    generate_response(response)
}
