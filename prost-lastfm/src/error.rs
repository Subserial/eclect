// Re-export error types.

mod proto {
    include!(concat!(env!("OUT_DIR"), "/lastfm.error.rs"));
}

pub use proto::error::Error as ErrorCode;
use std::fmt::{Display, Formatter};

#[derive(Debug, serde::Deserialize)]
pub struct LastFMError {
    pub message: String,
    pub error: ErrorCode,
}

impl Display for LastFMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error, self.message)
    }
}

#[derive(Debug, into_enum::IntoEnum)]
pub enum Error {
    Reqwest(reqwest::Error),
    LastFM(LastFMError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Reqwest(err) => write!(f, "Error(Reqwest) {{ {} }}", err),
            Error::LastFM(err) => write!(f, "Error(LastFM) {{ {} }}", err),
        }
    }
}

impl std::error::Error for Error {}
