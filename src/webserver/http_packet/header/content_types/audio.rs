use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AudioSubType {
    Mpeg,
    Mp4,
    Ogg,
    Webm,
    Aac,
    Wav,
    Flac,
    Other(String),
}

impl FromStr for AudioSubType {
    type Err = ();

    fn from_str(sub: &str) -> Result<Self, Self::Err> {
        Ok(match sub {
            "mpeg" | "mp3" => AudioSubType::Mpeg,
            "mp4" => AudioSubType::Mp4,
            "ogg" => AudioSubType::Ogg,
            "webm" => AudioSubType::Webm,
            "aac" => AudioSubType::Aac,
            "wav" => AudioSubType::Wav,
            "flac" => AudioSubType::Flac,
            other => AudioSubType::Other(other.into()),
        })
    }
}
