use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VideoSubType {
    Mp4,
    Mpeg,
    Webm,
    Ogg,
    H264,
    H265,
    Other(String),
}

impl FromStr for VideoSubType {
    type Err = ();

    fn from_str(sub: &str) -> Result<Self, Self::Err> {
        Ok(match sub {
            "mp4" => VideoSubType::Mp4,
            "mpeg" => VideoSubType::Mpeg,
            "webm" => VideoSubType::Webm,
            "ogg" => VideoSubType::Ogg,
            "h264" => VideoSubType::H264,
            "h265" => VideoSubType::H265,
            other => VideoSubType::Other(other.into()),
        })
    }
}
