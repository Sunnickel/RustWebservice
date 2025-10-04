mod response_codes;

use crate::webserver::cookie::Cookie;
pub use crate::webserver::responses::response_codes::ResponseCodes;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Response {
    pub headers: ResponseHeaders,
    pub content: Arc<String>,
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

    pub fn add_cookie(&mut self, cookie: Cookie) {
        self.headers.set_cookie(cookie);
    }

    pub fn expire_cookie(&mut self, cookie: Cookie) {
        self.headers.expire_cookie(cookie);
    }

    pub fn update_content(mut self, content: Arc<String>) -> Self {
        self.content = content;
        self
    }
}

#[derive(Clone)]
pub struct ResponseHeaders {
    pub(crate) protocol: String,
    pub(crate) status: ResponseCodes,
    pub(crate) values: HashMap<String, String>,
}

impl ResponseHeaders {
    pub(crate) fn new(
        protocol: String,
        status: ResponseCodes,
        values: HashMap<String, String>,
    ) -> Self {
        Self {
            protocol,
            status,
            values,
        }
    }

    pub(crate) fn as_str(&self) -> String {
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

    pub(crate) fn get_status_code(&self) -> u16 {
        self.status.as_u16()
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn get_header(&self, header: &str) -> Option<String> {
        if self.values.contains_key(header) {
            Option::from(self.values.get(header).unwrap().to_string())
        } else {
            None
        }
    }

    pub fn set_cookie(&mut self, cookie: Cookie) {
        self.values
            .insert("Set-Cookie".to_string(), cookie.as_string());
    }

    pub fn expire_cookie(&mut self, cookie: Cookie) {
        let cookie = cookie.clone().expires(Some(
            DateTime::from_timestamp(0, 0).unwrap().timestamp() as u64,
        ));
        self.values
            .insert("Set-Cookie".to_string(), cookie.as_string());
    }
}
