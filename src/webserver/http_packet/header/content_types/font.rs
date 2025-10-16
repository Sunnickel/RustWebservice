use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FontSubType {
    Woff,
    Woff2,
    Otf,
    Ttf,
    Other(String),
}

impl FromStr for FontSubType {
    type Err = ();

    fn from_str(sub: &str) -> Result<Self, Self::Err> {
        Ok(match sub {
            "woff" => FontSubType::Woff,
            "woff2" => FontSubType::Woff2,
            "otf" => FontSubType::Otf,
            "ttf" => FontSubType::Ttf,
            other => FontSubType::Other(other.into()),
        })
    }
}
