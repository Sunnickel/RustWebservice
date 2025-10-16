use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApplicationSubType {
    Json,
    Xml,
    OctetStream,
    Pdf,
    Zip,
    Gzip,
    XWwwFormUrlEncoded,
    Wasm,
    Javascript,
    Other(String),
}

impl FromStr for ApplicationSubType {
    type Err = ();

    fn from_str(sub: &str) -> Result<Self, Self::Err> {
        Ok(match sub {
            "json" => ApplicationSubType::Json,
            "xml" => ApplicationSubType::Xml,
            "octet-stream" => ApplicationSubType::OctetStream,
            "pdf" => ApplicationSubType::Pdf,
            "zip" => ApplicationSubType::Zip,
            "gzip" => ApplicationSubType::Gzip,
            "x-www-form-urlencoded" => ApplicationSubType::XWwwFormUrlEncoded,
            "wasm" => ApplicationSubType::Wasm,
            "javascript" | "js" => ApplicationSubType::Javascript,
            other => ApplicationSubType::Other(other.into()),
        })
    }
}
