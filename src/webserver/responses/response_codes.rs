#[derive(Clone, Copy, Debug)]
#[repr(u16)]
pub enum ResponseCodes {
    // 1xx Informational
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,

    // 2xx Success
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NoContent = 204,

    // 3xx Redirection
    MovedPermanently = 301,
    Found = 302,
    NotModified = 304,

    // 4xx Client Error
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,

    // 5xx Server Error
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
}

impl ResponseCodes {
    pub fn as_str(&self) -> &'static str {
        match self {
            // 1xx
            ResponseCodes::Continue => "Continue",
            ResponseCodes::SwitchingProtocols => "Switching Protocols",
            ResponseCodes::Processing => "Processing",

            // 2xx
            ResponseCodes::Ok => "OK",
            ResponseCodes::Created => "Created",
            ResponseCodes::Accepted => "Accepted",
            ResponseCodes::NoContent => "No Content",

            // 3xx
            ResponseCodes::MovedPermanently => "Moved Permanently",
            ResponseCodes::Found => "Found",
            ResponseCodes::NotModified => "Not Modified",

            // 4xx
            ResponseCodes::BadRequest => "Bad Request",
            ResponseCodes::Unauthorized => "Unauthorized",
            ResponseCodes::Forbidden => "Forbidden",
            ResponseCodes::NotFound => "Not Found",
            ResponseCodes::MethodNotAllowed => "Method Not Allowed",

            // 5xx
            ResponseCodes::InternalServerError => "Internal Server Error",
            ResponseCodes::NotImplemented => "Not Implemented",
            ResponseCodes::BadGateway => "Bad Gateway",
            ResponseCodes::ServiceUnavailable => "Service Unavailable",
            ResponseCodes::GatewayTimeout => "Gateway Timeout",
        }
    }

    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}
