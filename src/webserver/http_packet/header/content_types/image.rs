use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageSubType {
    Png,
    Jpeg,
    Gif,
    Webp,
    SvgXml,
    Avif,
    Bmp,
    Other(String),
}

impl FromStr for ImageSubType {
    type Err = ();

    fn from_str(sub: &str) -> Result<Self, Self::Err> {
        Ok(match sub {
            "png" => ImageSubType::Png,
            "jpeg" | "jpg" => ImageSubType::Jpeg,
            "gif" => ImageSubType::Gif,
            "webp" => ImageSubType::Webp,
            "svg+xml" => ImageSubType::SvgXml,
            "avif" => ImageSubType::Avif,
            "bmp" => ImageSubType::Bmp,
            other => ImageSubType::Other(other.into()),
        })
    }
}
