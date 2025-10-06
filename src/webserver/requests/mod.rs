use crate::webserver::cookie::Cookie;
use crate::webserver::route::HTTPMethod;
use crate::webserver::Domain;
use std::collections::HashMap;

/// Represents an HTTP request received by the web server.
///
/// # Description
///
/// The `Request` struct holds all relevant information about an incoming HTTP request,
/// including protocol, method, route, headers, and remote address.
#[derive(Clone)]
pub struct Request {
    /// The HTTP protocol version (e.g., "HTTP/1.1").
    pub protocol: String,
    /// The HTTP method (e.g., "GET", "POST").
    pub method: HTTPMethod,
    /// The requested route/path.
    pub route: String,
    /// A map of header key-value pairs.
    pub values: HashMap<String, String>,
    /// The IP address of the client making the request.
    pub remote_addr: String,
}

impl Request {
    /// Creates a new `Request` instance from a raw HTTP request string.
    ///
    /// # Description
    ///
    /// Parses the raw HTTP request string and extracts the method, route, protocol,
    /// and headers. The request line is parsed to determine the components of the request.
    ///
    /// # Arguments
    ///
    /// * `request`: A string slice containing the full HTTP request.
    /// * `remote_addr`: A string slice representing the IP address of the client.
    ///
    /// # Returns
    ///
    /// An `Option<Request>` which is `Some(Request)` if parsing was successful,
    /// or `None` if the request line could not be parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// use webserver::requests::Request;
    ///
    /// let raw_request = "GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    /// let request = Request::new(raw_request.to_string(), "127.0.0.1".to_string());
    /// assert!(request.is_some());
    /// ```
    pub(crate) fn new(request: String, remote_addr: String) -> Option<Request> {
        let mut lines = request.lines();

        let mut method_str = String::new();
        let mut route = String::new();
        let mut protocol = String::new();

        let request_line = lines.next()?; // first line e.g. "GET /index.html HTTP/1.1"

        for part in request_line.split_whitespace() {
            let upper = part.to_uppercase();
            if upper.starts_with("HTTP/") || upper.starts_with("HTTPS/") {
                protocol = part.to_string();
            } else if part.starts_with('/')
                || part.starts_with("http://")
                || part.starts_with("https://")
            {
                route = part.to_string();
            } else {
                method_str = part.to_string();
            }
        }

        // Convert the method string to HTTPMethod enum
        let method = match method_str.as_str() {
            "GET" => HTTPMethod::GET,
            "POST" => HTTPMethod::POST,
            "PUT" => HTTPMethod::PUT,
            "DELETE" => HTTPMethod::DELETE,
            "PATCH" => HTTPMethod::PATCH,
            "OPTIONS" => HTTPMethod::OPTIONS,
            "HEAD" => HTTPMethod::HEAD,
            _ => HTTPMethod::GET,
        };

        let values: HashMap<String, String> = lines
            .filter_map(|line| line.split_once(':'))
            .map(|(key, value)| (key.trim().to_lowercase(), value.trim().to_string()))
            .collect();

        Some(Self {
            protocol,
            method,
            route,
            values,
            remote_addr,
        })
    }

    /// Retrieves all cookies from the request's "Cookie" header.
    ///
    /// # Description
    ///
    /// Parses the "Cookie" header value and constructs a vector of `Cookie` objects.
    /// Each cookie is initialized with the domain obtained from the "Host" header.
    ///
    /// # Returns
    ///
    /// A vector of `Cookie` structs parsed from the request headers.
    ///
    /// # Examples
    ///
    /// ```
    /// use webserver::requests::Request;
    ///
    /// let raw_request = "GET /index.html HTTP/1.1\r\nHost: example.com\r\nCookie: session=abc123; user=john\r\n\r\n";
    /// let request = Request::new(raw_request.to_string(), "127.0.0.1".to_string()).unwrap();
    /// let cookies = request.get_cookies();
    /// assert_eq!(cookies.len(), 2);
    /// ```
    pub fn get_cookies(&self) -> Vec<Cookie> {
        let Some(cookie_str) = self.values.get("cookie") else {
            return Vec::new();
        };
        let mut cookies: Vec<Cookie> = Vec::new();
        for cookie_pair in cookie_str.as_str().split(';') {
            if let Some((key, value)) = cookie_pair.trim().split_once('=') {
                if let Some(host) = self.values.get("host") {
                    cookies.push(Cookie::new(
                        key.trim(),
                        value.trim(),
                        &Domain::new(host.as_str()),
                    ));
                }
            }
        }
        cookies
    }

    /// Retrieves a specific cookie by its key from the request.
    ///
    /// # Description
    ///
    /// This method uses `get_cookies()` to retrieve all cookies and then filters
    /// for the one matching the provided key.
    ///
    /// # Arguments
    ///
    /// * `key`: A string slice representing the name of the cookie to find.
    ///
    /// # Returns
    ///
    /// An `Option<Cookie>` containing the cookie if found, or `None` if not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use webserver::requests::Request;
    ///
    /// let raw_request = "GET /index.html HTTP/1.1\r\nHost: example.com\r\nCookie: session=abc123; user=john\r\n\r\n";
    /// let request = Request::new(raw_request.to_string(), "127.0.0.1".to_string()).unwrap();
    /// let session_cookie = request.get_cookie("session");
    /// assert!(session_cookie.is_some());
    /// ```
    pub fn get_cookie(&self, key: &str) -> Option<Cookie> {
        self.get_cookies().into_iter().find(|c| c.key == key)
    }
}
