use crate::webserver::http_packet::header::content_types::audio::AudioSubType;
use crate::webserver::http_packet::header::content_types::text::TextSubType;
use crate::webserver::http_packet::header::content_types::video::VideoSubType;
use crate::webserver::http_packet::header::content_types::ContentType;
use crate::webserver::http_packet::header::headers::frame_option::FrameOption;
use crate::webserver::http_packet::header::HTTPHeader;
use crate::webserver::http_packet::HTTPMessage;
pub(crate) use crate::webserver::responses::status_code::StatusCode;
use std::collections::HashMap;

pub mod status_code;

#[derive(Clone, Debug)]
pub struct HTTPResponse {
    pub status_code: StatusCode,
    pub(crate) message: HTTPMessage,
}

// Create Responses
impl HTTPResponse {
    pub fn new(status_code: StatusCode) -> Self {
        let headers = HTTPHeader::new(HashMap::new());
        let message = HTTPMessage::new("HTTP/1.1".to_string(), headers);

        Self {
            status_code,
            message,
        }
    }

    pub(crate) fn ok() -> Self {
        Self::new(StatusCode::Ok)
    }

    /// Creates a 404 Not Found response
    pub fn not_found() -> Self {
        Self::new(StatusCode::NotFound)
    }

    /// Creates a 500 Internal Server Error response
    pub fn internal_error() -> Self {
        Self::new(StatusCode::InternalServerError)
    }

    /// Creates a 405 Method Not Allowed response
    pub(crate) fn method_not_allowed() -> Self {
        Self::new(StatusCode::MethodNotAllowed)
    }

    /// Creates a 405 Method Not Allowed response
    pub(crate) fn bad_gateway() -> Self {
        Self::new(StatusCode::BadGateway)
    }

    /// Creates a redirect response
    pub fn redirect(location: &str, permanent: bool) -> Self {
        let status = if permanent {
            StatusCode::TemporaryRedirect
        } else {
            StatusCode::PermanentRedirect
        };
        let mut response = Self::new(status);
        response.set_location(location);
        response
    }
}

// Functions
impl HTTPResponse {
    // ===== Header Delegation Methods =====

    /// Adds a custom header
    pub fn add_header(&mut self, key: &str, value: &str) {
        self.message.headers.add_header(key, value);
    }

    /// Gets a header value
    pub fn get_header(&self, name: &str) -> Option<String> {
        self.message.headers.get_header(name)
    }

    /// Gets all headers as a reference
    pub fn headers(&mut self) -> &mut HTTPHeader {
        &mut self.message.headers
    }

    /// Gets mutable access to headers
    pub fn headers_mut(&mut self) -> &mut HTTPHeader {
        &mut self.message.headers
    }

    // ===== Body Methods =====

    /// Sets the response body from bytes
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.message.headers.content_length = Some(body.len() as u64);
        self.message.body = Some(body);
    }

    /// Sets the response body from a string
    pub fn set_body_string(&mut self, body: String) {
        self.set_body(body.into_bytes());
    }

    /// Gets the response body
    pub fn body(&self) -> Option<&[u8]> {
        self.message.body.as_deref()
    }

    // ===== Convenience Methods (delegating to HTTPHeader) =====

    pub fn set_date_now(&mut self) {
        self.message.headers.set_date_now();
    }

    pub fn set_server(&mut self, server_name: &str) {
        self.message.headers.set_server(server_name);
    }

    pub fn set_location(&mut self, url: &str) {
        self.message.headers.set_location(url);
    }

    pub fn set_cache_control(&mut self, directive: &str) {
        self.message.headers.set_cache_control(directive);
    }

    pub fn set_no_cache(&mut self) {
        self.message.headers.set_no_cache();
    }

    pub fn set_max_age(&mut self, seconds: u64) {
        self.message.headers.set_max_age(seconds);
    }

    pub fn set_etag(&mut self, etag: &str) {
        self.message.headers.set_etag(etag);
    }

    pub fn set_content_encoding(&mut self, encoding: &str) {
        self.message.headers.set_content_encoding(encoding);
    }

    pub fn set_transfer_encoding(&mut self, encoding: &str) {
        self.message.headers.set_transfer_encoding(encoding);
    }

    // Security headers
    pub fn set_nosniff(&mut self) {
        self.message.headers.set_nosniff();
    }

    pub fn set_frame_options(&mut self, option: FrameOption) {
        self.message.headers.set_frame_options(option);
    }

    pub fn set_hsts(&mut self, max_age_seconds: u64, include_subdomains: bool) {
        self.message
            .headers
            .set_hsts(max_age_seconds, include_subdomains);
    }

    pub fn set_csp(&mut self, policy: &str) {
        self.message.headers.set_csp(policy);
    }

    pub fn set_xss_protection(&mut self, enabled: bool) {
        self.message.headers.set_xss_protection(enabled);
    }

    pub fn apply_security_headers(&mut self) {
        self.message.headers.apply_security_headers();
    }

    // CORS headers
    pub fn set_cors_origin(&mut self, origin: &str) {
        self.message.headers.set_cors_origin(origin);
    }

    pub fn set_cors_methods(&mut self, methods: &[&str]) {
        self.message.headers.set_cors_methods(methods);
    }

    pub fn set_cors_headers(&mut self, headers: &[&str]) {
        self.message.headers.set_cors_headers(headers);
    }

    pub fn set_cors_max_age(&mut self, seconds: u64) {
        self.message.headers.set_cors_max_age(seconds);
    }

    pub fn set_cors_credentials(&mut self, allow: bool) {
        self.message.headers.set_cors_credentials(allow);
    }

    pub fn apply_cors_permissive(&mut self) {
        self.message.headers.apply_cors_permissive();
    }

    // ===== Content-Type Methods =====

    /// Sets the content type of the response
    pub fn set_content_type(&mut self, content_type: ContentType) {
        self.message.headers.content_type = content_type;
    }

    /// Gets the current content type
    pub fn content_type(&self) -> &ContentType {
        &self.message.headers.content_type
    }

    /// Convenience method: Set content type to JSON
    pub fn set_json(&mut self) {
        use crate::webserver::http_packet::header::content_types::{
            application::ApplicationSubType, ContentType,
        };
        self.set_content_type(ContentType::Application(ApplicationSubType::Json));
    }

    /// Convenience method: Set content type to HTML
    pub fn set_html(&mut self) {
        self.set_content_type(ContentType::Text(TextSubType::Html));
    }

    /// Convenience method: Set content type to plain text
    pub fn set_text(&mut self) {
        use crate::webserver::http_packet::header::content_types::{
            text::TextSubType, ContentType,
        };
        self.set_content_type(ContentType::Text(TextSubType::Plain));
    }

    /// Convenience method: Set content type to video
    pub fn set_video(&mut self, subtype: VideoSubType) {
        self.set_content_type(ContentType::Video(subtype));
    }

    /// Convenience method: Set content type to audio
    pub fn set_audio(&mut self, subtype: AudioSubType) {
        self.set_content_type(ContentType::Audio(subtype));
    }

    /// Convenience method: Set content type to image
    pub fn set_image(
        &mut self,
        subtype: crate::webserver::http_packet::header::content_types::image::ImageSubType,
    ) {
        use crate::webserver::http_packet::header::content_types::ContentType;
        self.set_content_type(ContentType::Image(subtype));
    }

    // ===== Response Building Methods =====

    /// Converts the response to bytes for sending over the network
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut response = format!(
            "{} {} {}\r\n",
            self.message.http_version,
            self.status_code.as_u16(),
            self.status_code.to_string()
        );

        // Add content-type and content-length
        response.push_str(&format!(
            "Content-Type: {}\r\n",
            self.message.headers.content_type.to_string()
        ));

        if let Some(len) = self.message.headers.content_length {
            response.push_str(&format!("Content-Length: {}\r\n", len));
        }

        response.push_str(&format!(
            "Connection: {}\r\n",
            self.message.headers.connection.to_string()
        ));

        // Add all other headers
        response.push_str(&self.message.headers.as_str());

        // End of headers
        response.push_str("\r\n");

        let mut bytes = response.into_bytes();

        // Add body if present
        if let Some(body) = &self.message.body {
            bytes.extend_from_slice(body);
        }

        bytes
    }
}
