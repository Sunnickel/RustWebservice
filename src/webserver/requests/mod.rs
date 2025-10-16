use crate::webserver::http_packet::header::content_types::ContentType;
use crate::webserver::http_packet::header::headers::cookie::Cookie;
use crate::webserver::http_packet::header::HTTPHeader;
use crate::webserver::http_packet::HTTPMessage;
use crate::webserver::route::HTTPMethod;
use crate::webserver::Domain;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct HTTPRequest {
    pub method: HTTPMethod,
    pub path: String,
    pub(crate) message: HTTPMessage,
    pub query_params: HashMap<String, String>,
    pub path_params: HashMap<String, String>,
    pub form_params: HashMap<String, String>,
    pub cookie_jar: Vec<Cookie>,
}

impl HTTPRequest {
    pub(crate) fn new(method: HTTPMethod, path: String, message: HTTPMessage) -> Self {
        let mut request = Self {
            method,
            path,
            message,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
            form_params: HashMap::new(),
            cookie_jar: Vec::new(),
        };

        request.parse_query_params();
        request.parse_cookies();
        request
    }

    pub fn parse(raw_request: &[u8]) -> Result<Self, String> {
        let request_str = String::from_utf8(raw_request.to_vec())
            .map_err(|e| format!("Invalid UTF-8 in request: {}", e))?;

        let mut lines = request_str.lines();

        let request_line = lines.next().ok_or("Empty request")?;
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err("Invalid request line format".to_string());
        }

        let method = HTTPMethod::from_str(parts[0])
            .map_err(|_| format!("Unknown HTTP method: {}", parts[0]))?;
        let path = parts[1].to_string();
        let http_version = parts[2].to_string();

        let mut header_map = HashMap::new();

        for line in &mut lines {
            if line.is_empty() {
                break;
            }
            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                header_map.insert(name, value);
            }
        }

        let headers = HTTPHeader::new(header_map);

        // Parse body if Content-Length is present
        let body = if let Ok(Some(content_length_str)) = headers
            .get_header("Content-Length")
            .ok_or("No content length")
            .map(|h| Some(h))
        {
            if let Ok(content_length) = usize::from_str(&content_length_str) {
                let remaining = request_str
                    .lines()
                    .last()
                    .map(|l| l.as_bytes())
                    .unwrap_or(&[]);
                if remaining.len() >= content_length {
                    Some(remaining[..content_length].to_vec())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let message = HTTPMessage {
            http_version,
            headers,
            body,
        };

        let mut request = Self {
            method,
            path,
            message,
            query_params: HashMap::new(),
            path_params: HashMap::new(),
            form_params: HashMap::new(),
            cookie_jar: Vec::new(),
        };

        request.parse_query_params();
        request.parse_cookies();
        request.parse_form_params();

        Ok(request)
    }

    // ===== Basic Properties =====
    pub fn path(&self) -> &str {
        &self.path
    }

    // ===== Header Operations =====
    pub fn get_header(&self, name: &str) -> Option<String> {
        self.message.headers.get_header(name)
    }

    pub fn headers(&self) -> &HTTPHeader {
        &self.message.headers
    }

    pub fn has_header(&self, name: &str) -> bool {
        self.message.headers.get_header(name).is_some()
    }

    pub fn content_type(&self) -> Option<ContentType> {
        Some(self.get_content_type())
    }

    pub fn content_length(&self) -> Option<usize> {
        self.get_header("Content-Length")
            .and_then(|s| usize::from_str(&s).ok())
    }

    pub fn user_agent(&self) -> Option<String> {
        self.get_header("User-Agent")
    }

    pub fn host(&self) -> Option<String> {
        self.get_header("Host")
    }

    pub fn authorization(&self) -> Option<String> {
        self.get_header("Authorization")
    }

    // ===== Body Operations =====

    pub fn body(&self) -> Option<&[u8]> {
        self.message.body.as_deref()
    }

    pub fn body_string(&self) -> Option<String> {
        self.message
            .body
            .as_ref()
            .and_then(|b| String::from_utf8(b.clone()).ok())
    }

    pub fn body_bytes(&self) -> Option<Vec<u8>> {
        self.message.body.clone()
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.message.body = Some(body);
    }

    pub fn get_content_type(&self) -> ContentType {
        self.content_type().unwrap()
    }

    // ===== Query Parameters =====

    pub fn query_param(&self, key: &str) -> Option<String> {
        self.query_params.get(key).cloned()
    }

    pub fn query_param_int(&self, key: &str) -> Option<i64> {
        self.query_params
            .get(key)
            .and_then(|s| i64::from_str(s).ok())
    }

    pub fn query_param_float(&self, key: &str) -> Option<f64> {
        self.query_params
            .get(key)
            .and_then(|s| f64::from_str(s).ok())
    }

    pub fn query_param_bool(&self, key: &str) -> Option<bool> {
        self.query_params
            .get(key)
            .and_then(|s| match s.to_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => bool::from_str(s).ok(),
            })
    }

    pub fn query_param_or(&self, key: &str, default: &str) -> String {
        self.query_params
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    pub fn all_query_params(&self) -> &HashMap<String, String> {
        &self.query_params
    }

    // ===== Path Parameters =====

    pub fn path_param(&self, key: &str) -> Option<String> {
        self.path_params.get(key).cloned()
    }

    pub fn path_param_int(&self, key: &str) -> Option<i64> {
        self.path_params
            .get(key)
            .and_then(|s| i64::from_str(s).ok())
    }

    pub fn set_path_param(&mut self, key: String, value: String) {
        self.path_params.insert(key, value);
    }

    pub fn all_path_params(&self) -> &HashMap<String, String> {
        &self.path_params
    }

    // ===== Form Parameters =====

    pub fn form_param(&self, key: &str) -> Option<String> {
        self.form_params.get(key).cloned()
    }

    pub fn form_param_int(&self, key: &str) -> Option<i64> {
        self.form_params
            .get(key)
            .and_then(|s| i64::from_str(s).ok())
    }

    pub fn all_form_params(&self) -> &HashMap<String, String> {
        &self.form_params
    }

    // ===== Cookies =====

    pub fn cookie(&self, name: &str) -> Option<Cookie> {
        Some(
            self.cookie_jar
                .iter()
                .map(|cookie: &Cookie| cookie.key == name)
                .collect(),
        )
    }

    pub fn all_cookies(&self) -> &Vec<Cookie> {
        &self.cookie_jar
    }

    pub fn has_cookie(&self, name: &str) -> bool {
        self.cookie(name).is_some()
    }

    pub fn has_body(&self) -> bool {
        self.message.body.is_some() && !self.message.body.as_ref().unwrap().is_empty()
    }

    // ===== Parsing Private Methods =====

    fn parse_query_params(&mut self) {
        if let Some(query_start) = self.path.find('?') {
            let query_string = &self.path[query_start + 1..];
            for pair in query_string.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = self.url_decode(&pair[..eq_pos]);
                    let value = self.url_decode(&pair[eq_pos + 1..]);
                    self.query_params.insert(key, value);
                } else {
                    self.query_params
                        .insert(self.url_decode(pair), String::new());
                }
            }
        }
    }

    fn parse_cookies(&mut self) {
        if let Some(cookie_header) = self.get_header("Cookie") {
            for cookie in cookie_header.split(';') {
                if let Some(eq_pos) = cookie.find('=') {
                    let key = cookie[..eq_pos].trim().to_string();
                    let value = cookie[eq_pos + 1..].trim().to_string();
                    self.cookie_jar.push(Cookie::new(
                        &*key,
                        &*value,
                        &Domain::new(self.host().unwrap().as_str()),
                    ));
                }
            }
        }
    }

    fn url_decode(&self, encoded: &str) -> String {
        let mut decoded = String::new();
        let mut chars = encoded.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '%' => {
                    if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                        if let Ok(byte) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                            decoded.push(byte as char);
                        }
                    }
                }
                '+' => decoded.push(' '),
                _ => decoded.push(ch),
            }
        }
        decoded
    }

    fn parse_form_params(&mut self) {
        if let Some(body) = &self.message.body {
            if let Ok(body_str) = String::from_utf8(body.clone()) {
                let content_type = self.get_header("Content-Type").unwrap_or_default();

                if content_type.contains("application/x-www-form-urlencoded") {
                    self.parse_url_encoded_form(&body_str);
                } else if content_type.contains("application/json") {
                    self.parse_json_form(&body_str);
                }
            }
        }
    }

    fn parse_url_encoded_form(&mut self, body: &str) {
        for pair in body.split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let key = self.url_decode(&pair[..eq_pos]);
                let value = self.url_decode(&pair[eq_pos + 1..]);
                self.form_params.insert(key, value);
            } else {
                self.form_params
                    .insert(self.url_decode(pair), String::new());
            }
        }
    }

    fn parse_json_form(&mut self, body: &str) {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body) {
            if let Some(obj) = json_value.as_object() {
                for (key, value) in obj {
                    let value_str = match value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Null => String::new(),
                        _ => value.to_string(),
                    };
                    self.form_params.insert(key.clone(), value_str);
                }
            }
        }
    }
}
