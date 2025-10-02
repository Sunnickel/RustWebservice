mod response_codes;

pub(crate) use crate::webserver::responses::response_codes::ResponseCodes;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Response {
    pub headers: ResponseHeaders,
    content: Arc<String>,
}

#[derive(Clone)]
pub struct ResponseHeaders {
    protocol: String,
    status: ResponseCodes,
    values: HashMap<String, String>,
}

impl ResponseHeaders {
    pub fn new(protocol: String, status: ResponseCodes, values: HashMap<String, String>) -> Self {
        Self {
            protocol,
            status,
            values,
        }
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn as_str(&self) -> String {
        let mut output = format!(
            "{} {:?} {}\r\n",
            self.protocol,
            self.status.as_u16(),
            self.status.as_str()
        );
        output.push_str(
            self.values
                .iter()
                .map(|(k, v)| format!("{}: {}\r\n", k, v))
                .collect::<String>()
                .as_str(),
        );
        output.push_str("\r\n");
        output
    }

    pub fn get_status_code(&self) -> u16 {
        self.status.as_u16()
    }
}

impl Response {
    pub fn new(
        content: Arc<String>,
        mut code: Option<ResponseCodes>,
        mut protocol: Option<String>,
    ) -> Response {
        if protocol.is_none() {
            protocol = Some(String::from("HTTP/2"))
        }
        if code.is_none() {
            code = Some(ResponseCodes::Ok);
        }

        let mut header_values = HashMap::new();
        header_values.insert(String::from("Date"), Utc::now().to_rfc2822());
        header_values.insert(String::from("Server"), String::from("RustWebServer/1.0"));
        header_values.insert(String::from("Content-Type"), String::from("text/html"));
        header_values.insert(String::from("Content-Length"), content.len().to_string());
        header_values.insert(String::from("Connection"), String::from("close"));

        let headers = ResponseHeaders::new(protocol.unwrap(), code.unwrap(), header_values);

        Self { headers, content }
    }

    pub fn as_str(&self) -> String {
        let mut output = String::from(self.headers.as_str());
        output.push_str(self.content.as_str());
        output
    }
}
