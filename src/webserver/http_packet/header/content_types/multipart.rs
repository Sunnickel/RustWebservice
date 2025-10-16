use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MultipartSubType {
    FormData,
    Mixed,
    Alternative,
    Related,
    Other(String),
}

impl FromStr for MultipartSubType {
    type Err = ();

    fn from_str(sub: &str) -> Result<Self, Self::Err> {
        Ok(match sub {
            "form-data" => MultipartSubType::FormData,
            "mixed" => MultipartSubType::Mixed,
            "alternative" => MultipartSubType::Alternative,
            "related" => MultipartSubType::Related,
            other => MultipartSubType::Other(other.into()),
        })
    }
}
