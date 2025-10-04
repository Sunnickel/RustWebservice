#[derive(Clone, Copy, Debug)]
#[repr(u16)]
pub enum ResponseCodes {
    // 1xx Informational
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,
    EarlyHints = 103,

    // 2xx Success
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    ImUsed = 226,

    // 3xx Redirection
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    // 4xx Client Error
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    ContentTooLarge = 413,
    UriTooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableContent = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,

    // 5xx Server Error
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 509,
    NetworkAuthenticationRequired = 510,
}

impl ResponseCodes {
    pub fn as_str(&self) -> &'static str {
        match self {
            // 1xx
            ResponseCodes::Continue => "Continue",
            ResponseCodes::SwitchingProtocols => "Switching Protocols",
            ResponseCodes::Processing => "Processing",
            ResponseCodes::EarlyHints => "Early Hints",

            // 2xx
            ResponseCodes::Ok => "OK",
            ResponseCodes::Created => "Created",
            ResponseCodes::Accepted => "Accepted",
            ResponseCodes::NonAuthoritativeInformation => "Non-Authoritative Information",
            ResponseCodes::NoContent => "No Content",
            ResponseCodes::ResetContent => "Reset Content",
            ResponseCodes::PartialContent => "Partial Content",
            ResponseCodes::MultiStatus => "Multi-Status",
            ResponseCodes::AlreadyReported => "Already Reported",
            ResponseCodes::ImUsed => "IM Used",

            // 3xx
            ResponseCodes::MultipleChoices => "Multiple Choices",
            ResponseCodes::MovedPermanently => "Moved Permanently",
            ResponseCodes::Found => "Found",
            ResponseCodes::SeeOther => "See Other",
            ResponseCodes::NotModified => "Not Modified",
            ResponseCodes::TemporaryRedirect => "Temporary Redirect",
            ResponseCodes::PermanentRedirect => "Permanent Redirect",

            // 4xx
            ResponseCodes::BadRequest => "Bad Request",
            ResponseCodes::Unauthorized => "Unauthorized",
            ResponseCodes::PaymentRequired => "Payment Required",
            ResponseCodes::Forbidden => "Forbidden",
            ResponseCodes::NotFound => "Not Found",
            ResponseCodes::MethodNotAllowed => "Method Not Allowed",
            ResponseCodes::NotAcceptable => "Not Acceptable",
            ResponseCodes::ProxyAuthenticationRequired => "Proxy Authentication Required",
            ResponseCodes::RequestTimeout => "Request Timeout",
            ResponseCodes::Conflict => "Conflict",
            ResponseCodes::Gone => "Gone",
            ResponseCodes::LengthRequired => "Length Required",
            ResponseCodes::PreconditionFailed => "Precondition Failed",
            ResponseCodes::ContentTooLarge => "Content Too Large",
            ResponseCodes::UriTooLong => "URI Too Long",
            ResponseCodes::UnsupportedMediaType => "Unsupported Media Type",
            ResponseCodes::RangeNotSatisfiable => "Range Not Satisfiable",
            ResponseCodes::ExpectationFailed => "Expectation Failed",
            ResponseCodes::ImATeapot => "I'm a teapot",
            ResponseCodes::MisdirectedRequest => "Misdirected Request",
            ResponseCodes::UnprocessableContent => "Unprocessable Content",
            ResponseCodes::Locked => "Locked",
            ResponseCodes::FailedDependency => "Failed Dependency",
            ResponseCodes::TooEarly => "Too Early",
            ResponseCodes::UpgradeRequired => "Upgrade Required",
            ResponseCodes::PreconditionRequired => "Precondition Required",
            ResponseCodes::TooManyRequests => "Too Many Requests",
            ResponseCodes::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            ResponseCodes::UnavailableForLegalReasons => "Unavailable For Legal Reasons",

            // 5xx
            ResponseCodes::InternalServerError => "Internal Server Error",
            ResponseCodes::NotImplemented => "Not Implemented",
            ResponseCodes::BadGateway => "Bad Gateway",
            ResponseCodes::ServiceUnavailable => "Service Unavailable",
            ResponseCodes::GatewayTimeout => "Gateway Timeout",
            ResponseCodes::HTTPVersionNotSupported => "HTTP Version Not Supported",
            ResponseCodes::VariantAlsoNegotiates => "Variant Also Negotiates",
            ResponseCodes::InsufficientStorage => "Insufficient Storage",
            ResponseCodes::LoopDetected => "Loop Detected",
            ResponseCodes::NotExtended => "Not Extended",
            ResponseCodes::NetworkAuthenticationRequired => "Network Authentication Required",
        }
    }

    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}
