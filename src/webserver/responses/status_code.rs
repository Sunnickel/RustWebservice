//! HTTP response status codes enumeration
//!
//! This module defines all standard HTTP response status codes as an enumeration,
//! organized by category (1xx Informational, 2xx Success, 3xx Redirection,
//! 4xx Client Error, and 5xx Server Error). Each variant corresponds to a specific
//! HTTP status code and includes both the numeric value and descriptive string.
//!
//! The enum implements conversion methods to retrieve the numeric code and
//! descriptive string representation of each status code.
//!
//! # Examples
//!
//! ```
//! use your_crate::ResponseCodes;
//!
//! let status = ResponseCodes::Ok;
//! assert_eq!(status.as_u16(), 200);
//! assert_eq!(status.as_str(), "OK");
//!
//! let not_found = ResponseCodes::NotFound;
//! assert_eq!(not_found.as_u16(), 404);
//! assert_eq!(not_found.as_str(), "Not Found");
//! ```

use std::fmt;
use std::fmt::Formatter;

/// HTTP response status codes enum
///
/// This enum represents all standard HTTP response status codes organized by class:
/// - 1xx: Informational
/// - 2xx: Success
/// - 3xx: Redirection
/// - 4xx: Client Error
/// - 5xx: Server Error
///
/// Each variant is explicitly assigned its corresponding numeric value to match
/// the HTTP status code standard. The enum uses `u16` representation to accommodate
/// all possible HTTP status codes.
///
/// # Examples
/// ```
/// let status = ResponseCodes::Ok;
/// assert_eq!(status as u16, 200);
///
/// let status = ResponseCodes::NotFound;
/// assert_eq!(status as u16, 404);
/// ```
#[derive(Clone, Copy, Debug)]
#[repr(u16)]
#[derive(Eq, Hash, PartialEq)]
pub enum StatusCode {
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

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                // 1xx
                StatusCode::Continue => "Continue",
                StatusCode::SwitchingProtocols => "Switching Protocols",
                StatusCode::Processing => "Processing",
                StatusCode::EarlyHints => "Early Hints",

                // 2xx
                StatusCode::Ok => "OK",
                StatusCode::Created => "Created",
                StatusCode::Accepted => "Accepted",
                StatusCode::NonAuthoritativeInformation => "Non-Authoritative Information",
                StatusCode::NoContent => "No Content",
                StatusCode::ResetContent => "Reset Content",
                StatusCode::PartialContent => "Partial Content",
                StatusCode::MultiStatus => "Multi-Status",
                StatusCode::AlreadyReported => "Already Reported",
                StatusCode::ImUsed => "IM Used",

                // 3xx
                StatusCode::MultipleChoices => "Multiple Choices",
                StatusCode::MovedPermanently => "Moved Permanently",
                StatusCode::Found => "Found",
                StatusCode::SeeOther => "See Other",
                StatusCode::NotModified => "Not Modified",
                StatusCode::TemporaryRedirect => "Temporary Redirect",
                StatusCode::PermanentRedirect => "Permanent Redirect",

                // 4xx
                StatusCode::BadRequest => "Bad Request",
                StatusCode::Unauthorized => "Unauthorized",
                StatusCode::PaymentRequired => "Payment Required",
                StatusCode::Forbidden => "Forbidden",
                StatusCode::NotFound => "Not Found",
                StatusCode::MethodNotAllowed => "Method Not Allowed",
                StatusCode::NotAcceptable => "Not Acceptable",
                StatusCode::ProxyAuthenticationRequired => "Proxy Authentication Required",
                StatusCode::RequestTimeout => "Request Timeout",
                StatusCode::Conflict => "Conflict",
                StatusCode::Gone => "Gone",
                StatusCode::LengthRequired => "Length Required",
                StatusCode::PreconditionFailed => "Precondition Failed",
                StatusCode::ContentTooLarge => "Content Too Large",
                StatusCode::UriTooLong => "URI Too Long",
                StatusCode::UnsupportedMediaType => "Unsupported Media Type",
                StatusCode::RangeNotSatisfiable => "Range Not Satisfiable",
                StatusCode::ExpectationFailed => "Expectation Failed",
                StatusCode::ImATeapot => "I'm a teapot",
                StatusCode::MisdirectedRequest => "Misdirected Request",
                StatusCode::UnprocessableContent => "Unprocessable Content",
                StatusCode::Locked => "Locked",
                StatusCode::FailedDependency => "Failed Dependency",
                StatusCode::TooEarly => "Too Early",
                StatusCode::UpgradeRequired => "Upgrade Required",
                StatusCode::PreconditionRequired => "Precondition Required",
                StatusCode::TooManyRequests => "Too Many Requests",
                StatusCode::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
                StatusCode::UnavailableForLegalReasons => "Unavailable For Legal Reasons",

                // 5xx
                StatusCode::InternalServerError => "Internal Server Error",
                StatusCode::NotImplemented => "Not Implemented",
                StatusCode::BadGateway => "Bad Gateway",
                StatusCode::ServiceUnavailable => "Service Unavailable",
                StatusCode::GatewayTimeout => "Gateway Timeout",
                StatusCode::HTTPVersionNotSupported => "HTTP Version Not Supported",
                StatusCode::VariantAlsoNegotiates => "Variant Also Negotiates",
                StatusCode::InsufficientStorage => "Insufficient Storage",
                StatusCode::LoopDetected => "Loop Detected",
                StatusCode::NotExtended => "Not Extended",
                StatusCode::NetworkAuthenticationRequired => "Network Authentication Required",
            }
        )
    }
}

impl StatusCode {
    ///
    /// # Arguments
    /// * `response_codes` - The other response code to compare against
    ///
    /// # Returns
    /// Returns `true` if both the numeric value and string representation match,
    /// `false` otherwise.
    ///
    /// # Examples
    /// ```
    /// use your_crate::ResponseCodes;
    ///
    /// let status1 = ResponseCodes::Ok;
    /// let status2 = ResponseCodes::Ok;
    /// assert_eq!(status1.equals(status2), true);
    ///
    /// let status3 = ResponseCodes::NotFound;
    /// assert_eq!(status1.equals(status3), false);
    /// ```
    pub fn equals(&self, response_codes: StatusCode) -> bool {
        self.as_u16() == response_codes.as_u16() && self.to_string() == response_codes.to_string()
    }

    /// Get the numeric value of the response code
    ///
    /// # Returns
    /// The u16 representation of this HTTP status code.
    ///
    /// # Examples
    /// ```
    /// use your_crate::ResponseCodes;
    ///
    /// let status = ResponseCodes::Ok;
    /// assert_eq!(status.as_u16(), 200);
    ///
    /// let not_found = ResponseCodes::NotFound;
    /// assert_eq!(not_found.as_u16(), 404);
    /// ```
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}
