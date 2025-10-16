use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConnectionType {
    KeepAlive,
    Close,
    Upgrade,
    Other(String),
}

impl fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ConnectionType::KeepAlive => "keep-alive",
                ConnectionType::Close => "close",
                ConnectionType::Upgrade => "upgrade",
                ConnectionType::Other(text) => text,
            }
        )
    }
}
