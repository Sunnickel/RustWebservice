use std::fmt;

#[derive(Clone, PartialEq, Debug)]
pub enum HTTPMethod {
    GET,
    HEAD,
    OPTIONS,
    TRACE,
    PUT,
    DELETE,
    POST,
    PATCH,
    CONNECT,
}

impl HTTPMethod {
    pub fn from_str(method: &str) -> Result<HTTPMethod, String> {
        match method.to_uppercase().as_str() {
            "GET" => Ok(HTTPMethod::GET),
            "HEAD" => Ok(HTTPMethod::HEAD),
            "OPTIONS" => Ok(HTTPMethod::OPTIONS),
            "TRACE" => Ok(HTTPMethod::TRACE),
            "PUT" => Ok(HTTPMethod::PUT),
            "DELETE" => Ok(HTTPMethod::DELETE),
            "POST" => Ok(HTTPMethod::POST),
            "PATCH" => Ok(HTTPMethod::PATCH),
            "CONNECT" => Ok(HTTPMethod::CONNECT),
            _ => Err(format!("Unknown HTTP method: {}", method)),
        }
    }
}

impl fmt::Display for HTTPMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HTTPMethod::GET => write!(f, "GET"),
            HTTPMethod::HEAD => write!(f, "HEAD"),
            HTTPMethod::OPTIONS => write!(f, "OPTIONS"),
            HTTPMethod::TRACE => write!(f, "TRACE"),
            HTTPMethod::PUT => write!(f, "PUT"),
            HTTPMethod::DELETE => write!(f, "DELETE"),
            HTTPMethod::POST => write!(f, "POST"),
            HTTPMethod::PATCH => write!(f, "PATCH"),
            HTTPMethod::CONNECT => write!(f, "CONNECT"),
            _ => write!(f, "UNKNOWN"),
        }
    }
}
