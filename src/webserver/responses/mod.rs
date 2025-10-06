//! Module for HTTP response handling
//!
//! This module provides the `Response` and `ResponseHeaders` structs for creating
//! and managing HTTP responses. It includes functionality for setting headers,
//! cookies, content, and formatting the complete response string.
//!
//! The `ResponseCodes` enum is re-exported from the `response_codes` module to
//! provide standard HTTP status codes.

mod response_codes;

use crate::webserver::cookie::Cookie;
pub use crate::webserver::responses::response_codes::ResponseCodes;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

/// Represents an HTTP response
///
/// The `Response` struct encapsulates all the components of an HTTP response,
/// including headers and content. It provides methods for creating responses,
/// adding cookies, and formatting the complete response as a string.
///
/// # Examples
/// ```
/// use your_crate::Response;
/// use std::sync::Arc;
///
/// let content = Arc::new(String::from("<html><body>Hello World</body></html>"));
/// let response = Response::new(content, None, None);
/// ```
#[derive(Clone, Debug)]
pub struct Response {
    pub headers: ResponseHeaders,
    pub content: Arc<String>,
}

impl Response {
    /// Creates a new HTTP response
    ///
    /// # Arguments
    /// * `content` - The response body content as an Arc`<String>`
    /// * `code` - Optional status code. If None, defaults to Ok (200)
    /// * `protocol` - Optional protocol version. If None, defaults to "HTTP/2"
    ///
    /// # Returns
    /// A new `Response` instance with default headers set
    ///
    /// # Examples
    /// ```
    /// use your_crate::Response;
    /// use your_crate::ResponseCodes;
    /// use std::sync::Arc;
    ///
    /// let content = Arc::new(String::from("Hello World"));
    /// let response = Response::new(content, Some(ResponseCodes::NotFound), None);
    /// ```
    pub fn new(
        content: Arc<String>,
        mut code: Option<ResponseCodes>,
        mut protocol: Option<String>,
    ) -> Response {
        if protocol.is_none() {
            protocol = Some(String::from("HTTP/1.1"))
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

    /// Converts the response to a string representation
    ///
    /// # Returns
    /// A complete HTTP response as a string including headers and body
    ///
    /// # Examples
    /// ```
    /// use your_crate::Response;
    /// use std::sync::Arc;
    ///
    /// let content = Arc::new(String::from("<html><body>Hello World</body></html>"));
    /// let response = Response::new(content, None, None);
    /// let response_string = response.as_str();
    /// ```
    pub fn as_str(&self) -> String {
        let mut output = String::from(self.headers.as_str());
        output.push_str(self.content.as_str());
        output
    }

    /// Adds a cookie to the response headers
    ///
    /// # Arguments
    /// * `cookie` - The cookie to add to the response
    ///
    /// # Examples
    /// ```
    /// use your_crate::Response;
    /// use your_crate::Cookie;
    /// use std::sync::Arc;
    ///
    /// let content = Arc::new(String::from("Hello World"));
    /// let mut response = Response::new(content, None, None);
    /// let cookie = Cookie::new("session", "abc123");
    /// response.add_cookie(cookie);
    /// ```
    pub fn add_cookie(&mut self, cookie: Cookie) {
        self.headers.set_cookie(cookie);
    }

    /// Expires a cookie in the response headers
    ///
    /// # Arguments
    /// * `cookie` - The cookie to expire
    ///
    /// # Examples
    /// ```
    /// use your_crate::Response;
    /// use your_crate::Cookie;
    /// use std::sync::Arc;
    ///
    /// let content = Arc::new(String::from("Hello World"));
    /// let mut response = Response::new(content, None, None);
    /// let cookie = Cookie::new("session", "abc123");
    /// response.expire_cookie(cookie);
    /// ```
    pub fn expire_cookie(&mut self, cookie: Cookie) {
        self.headers.expire_cookie(cookie);
    }

    /// Updates the content of the response
    ///
    /// # Arguments
    /// * `content` - `The new content as an Arc<String>`
    ///
    /// # Returns
    /// A new `Response` instance with updated content
    ///
    /// # Examples
    /// ```
    /// use your_crate::Response;
    /// use std::sync::Arc;
    ///
    /// let content1 = Arc::new(String::from("Hello World"));
    /// let mut response = Response::new(content1, None, None);
    /// let content2 = Arc::new(String::from("Goodbye World"));
    /// response = response.update_content(content2);
    /// ```
    pub fn update_content(mut self, content: Arc<String>) -> Self {
        self.content = content;
        self
    }
}

/// Represents HTTP response headers
///
/// The `ResponseHeaders` struct manages the HTTP protocol version, status code,
/// and header values for an HTTP response.
///
/// # Examples
/// ```
/// use your_crate::ResponseHeaders;
/// use your_crate::ResponseCodes;
/// use std::collections::HashMap;
///
/// let mut headers = ResponseHeaders::new(
///     "HTTP/2".to_string(),
///     ResponseCodes::Ok,
///     HashMap::new()
/// );
/// ```
#[derive(Clone, Debug)]
pub struct ResponseHeaders {
    pub(crate) protocol: String,
    pub(crate) status: ResponseCodes,
    pub(crate) values: HashMap<String, String>,
}

impl ResponseHeaders {
    /// Creates new response headers
    ///
    /// # Arguments
    /// * `protocol` - The HTTP protocol version (e.g., "HTTP/1.1")
    /// * `status` - The HTTP status code
    /// * `values` - A HashMap of header name-value pairs
    ///
    /// # Returns
    /// A new `ResponseHeaders` instance
    ///
    /// # Examples
    /// ```
    /// use your_crate::ResponseHeaders;
    /// use your_crate::ResponseCodes;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = ResponseHeaders::new(
    ///     "HTTP/2".to_string(),
    ///     ResponseCodes::Ok,
    ///     HashMap::new()
    /// );
    /// ```
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

    /// Converts the headers to a string representation
    ///
    /// # Returns
    /// A formatted string containing the HTTP status line and headers
    ///
    /// # Examples
    /// ```
    /// use your_crate::ResponseHeaders;
    /// use your_crate::ResponseCodes;
    /// use std::collections::HashMap;
    ///
    /// let headers = ResponseHeaders::new(
    ///     "HTTP/2".to_string(),
    ///     ResponseCodes::Ok,
    ///     HashMap::new()
    /// );
    /// let header_string = headers.as_str();
    /// ```
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

    /// Gets the status code as a u16 value
    ///
    /// # Returns
    /// The numeric HTTP status code
    ///
    /// # Examples
    /// ```
    /// use your_crate::ResponseHeaders;
    /// use your_crate::ResponseCodes;
    /// use std::collections::HashMap;
    ///
    /// let headers = ResponseHeaders::new(
    ///     "HTTP/2".to_string(),
    ///     ResponseCodes::NotFound,
    ///     HashMap::new()
    /// );
    /// assert_eq!(headers.get_status_code(), 404);
    /// ```
    pub(crate) fn get_status_code(&self) -> u16 {
        self.status.as_u16()
    }

    /// Adds a header to the response
    ///
    /// # Arguments
    /// * `key` - The header name
    /// * `value` - The header value
    ///
    /// # Examples
    /// ```
    /// use your_crate::ResponseHeaders;
    /// use your_crate::ResponseCodes;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = ResponseHeaders::new(
    ///     "HTTP/2".to_string(),
    ///     ResponseCodes::Ok,
    ///     HashMap::new()
    /// );
    /// headers.add_header("X-Custom-Header", "CustomValue");
    /// ```
    pub fn add_header(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    /// Gets a header value by name
    ///
    /// # Arguments
    /// * `header` - The name of the header to retrieve
    ///
    /// # Returns
    /// An Option containing the header value if found, None otherwise
    ///
    /// # Examples
    /// ```
    /// use your_crate::ResponseHeaders;
    /// use your_crate::ResponseCodes;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = ResponseHeaders::new(
    ///     "HTTP/2".to_string(),
    ///     ResponseCodes::Ok,
    ///     HashMap::new()
    /// );
    /// headers.add_header("X-Custom-Header", "CustomValue");
    /// assert_eq!(headers.get_header("X-Custom-Header"), Some("CustomValue".to_string()));
    /// ```
    pub fn get_header(&self, header: &str) -> Option<String> {
        if self.values.contains_key(header) {
            Option::from(self.values.get(header).unwrap().to_string())
        } else {
            None
        }
    }

    /// Sets a cookie in the response headers
    ///
    /// # Arguments
    /// * `cookie` - The cookie to set
    ///
    /// # Examples
    /// ```
    /// use your_crate::ResponseHeaders;
    /// use your_crate::Cookie;
    /// use your_crate::ResponseCodes;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = ResponseHeaders::new(
    ///     "HTTP/2".to_string(),
    ///     ResponseCodes::Ok,
    ///     HashMap::new()
    /// );
    /// let cookie = Cookie::new("session", "abc123");
    /// headers.set_cookie(cookie);
    /// ```
    pub fn set_cookie(&mut self, cookie: Cookie) {
        self.values
            .insert("Set-Cookie".to_string(), cookie.as_string());
    }

    /// Expires a cookie in the response headers
    ///
    /// # Arguments
    /// * `cookie` - The cookie to expire
    ///
    /// # Examples
    /// ```
    /// use your_crate::ResponseHeaders;
    /// use your_crate::Cookie;
    /// use your_crate::ResponseCodes;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = ResponseHeaders::new(
    ///     "HTTP/2".to_string(),
    ///     ResponseCodes::Ok,
    ///     HashMap::new()
    /// );
    /// let cookie = Cookie::new("session", "abc123");
    /// headers.expire_cookie(cookie);
    /// ```
    pub fn expire_cookie(&mut self, cookie: Cookie) {
        let cookie = cookie.clone().expires(Some(
            DateTime::from_timestamp(0, 0).unwrap().timestamp() as u64,
        ));
        self.values
            .insert("Set-Cookie".to_string(), cookie.as_string());
    }
}
