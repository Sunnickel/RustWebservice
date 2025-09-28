mod response_codes;

use crate::webserver::responses::response_codes::ResponseCodes;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Response {
    headers: HashMap<String, String>,
    content: Arc<String>,
    status_code: ResponseCodes,
}

impl Response {
    pub fn new(content: Arc<String>, code: Option<ResponseCodes>) -> Response {
        let mut headers = HashMap::new();
        headers.insert(String::from("Date"), Utc::now().to_rfc2822());
        headers.insert(String::from("Server"), String::from("RustWebServer/1.0"));
        headers.insert(String::from("Content-Type"), String::from("text/html"));
        headers.insert(String::from("Content-Length"), content.len().to_string());
        headers.insert(String::from("Connection"), String::from("close"));

        if code.is_none() {
            Self { headers, content, status_code: ResponseCodes::Ok }
        } else {
            Self { headers, content, status_code: code.unwrap() }
        }
    }

    pub fn header_to_string(&self) -> String {
        let mut output: String = String::from("");

        for (key, value) in &self.headers {
            output.push_str(&format!("{}: {}\r\n", key, value));
        }
        output.push_str("\r\n");
        output
    }

    pub fn set_status_code(&mut self, status_code: ResponseCodes) {
        self.headers.insert(String::from("HTTP/1.1"), format!("{} {}", status_code., status_code.as_str()));
    }
}

pub fn generate_response(response: &Response) -> String {
    let output =

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
