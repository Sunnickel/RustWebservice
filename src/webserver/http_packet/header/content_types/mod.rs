pub mod application;
pub mod audio;
pub mod font;
pub mod image;
pub mod multipart;
pub mod text;
pub mod video;

use crate::webserver::http_packet::header::content_types::application::ApplicationSubType;
use crate::webserver::http_packet::header::content_types::audio::AudioSubType;
use crate::webserver::http_packet::header::content_types::font::FontSubType;
use crate::webserver::http_packet::header::content_types::image::ImageSubType;
use crate::webserver::http_packet::header::content_types::multipart::MultipartSubType;
use crate::webserver::http_packet::header::content_types::text::TextSubType;
use crate::webserver::http_packet::header::content_types::video::VideoSubType;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContentType {
    Text(TextSubType),
    Application(ApplicationSubType),
    Image(ImageSubType),
    Audio(AudioSubType),
    Video(VideoSubType),
    Font(FontSubType),
    Multipart(MultipartSubType),
    Unknown(String, String),
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (main, sub) = match self {
            ContentType::Text(s) => ("text", s.to_string()),
            ContentType::Application(s) => ("application", s.to_string()),
            ContentType::Image(s) => ("image", s.to_string()),
            ContentType::Audio(s) => ("audio", s.to_string()),
            ContentType::Video(s) => ("video", s.to_string()),
            ContentType::Font(s) => ("font", s.to_string()),
            ContentType::Multipart(s) => ("multipart", s.to_string()),
            ContentType::Unknown(m, s) => (m.as_str(), s.clone()),
        };
        write!(f, "{}/{}", main, sub)
    }
}

impl FromStr for ContentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (main, sub) = s.split_once('/').unwrap_or(("unknown", "unknown"));
        Ok(match main {
            "text" => ContentType::Text(TextSubType::from_str(sub)?),
            "application" => ContentType::Application(ApplicationSubType::from_str(sub)?),
            "image" => ContentType::Image(ImageSubType::from_str(sub)?),
            "audio" => ContentType::Audio(AudioSubType::from_str(sub)?),
            "video" => ContentType::Video(VideoSubType::from_str(sub)?),
            "font" => ContentType::Font(FontSubType::from_str(sub)?),
            "multipart" => ContentType::Multipart(MultipartSubType::from_str(sub)?),
            other => ContentType::Unknown(other.into(), sub.into()),
        })
    }
}

/// ---------- Display impls for all subtypes ----------
macro_rules! impl_display {
    ($($t:ty),*) => {
        $(
            impl fmt::Display for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let s = match self {
                        Self::Other(v) => v.clone(),
                        _ => format!("{:?}", self).to_lowercase(),
                    };
                    write!(f, "{}", s.replace('_', "-"))
                }
            }
        )*
    };
}

impl_display!(
    TextSubType,
    ApplicationSubType,
    ImageSubType,
    AudioSubType,
    VideoSubType,
    FontSubType,
    MultipartSubType
);
