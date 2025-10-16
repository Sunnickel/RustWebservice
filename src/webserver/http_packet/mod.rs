pub mod header;

use header::HTTPHeader;

#[derive(Clone, Debug)]
pub(crate) struct HTTPMessage {
    pub http_version: String,
    pub headers: HTTPHeader,
    pub body: Option<Vec<u8>>,
}

impl HTTPMessage {
    pub(crate) fn new(http_version: String, headers: HTTPHeader) -> Self {
        Self {
            http_version,
            headers,
            body: None,
        }
    }
}
