use crate::webserver::http_packet::header::content_types::ContentType;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextSubType {
    Plain,
    Html,
    Css,
    Javascript,
    Csv,
    Xml,
    Markdown,
    Other(String),
}

impl FromStr for TextSubType {
    type Err = ();

    fn from_str(sub: &str) -> Result<Self, Self::Err> {
        Ok(match sub {
            "plain" => TextSubType::Plain,
            "html" => TextSubType::Html,
            "css" => TextSubType::Css,
            "javascript" | "js" => TextSubType::Javascript,
            "csv" => TextSubType::Csv,
            "xml" => TextSubType::Xml,
            "markdown" => TextSubType::Markdown,
            other => TextSubType::Other(other.into()),
        })
    }
}
