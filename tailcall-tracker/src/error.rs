use derive_more::{From, DebugCustom};
use reqwest::header::InvalidHeaderValue;
use std::fmt::Display;


#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Reqwest Error")]
    Reqwest(reqwest::Error),

    #[debug(fmt = "Invalid Header Value")]
    InvalidHeaderValue(InvalidHeaderValue),

    #[debug(fmt = "Serde JSON Error")]
    SerdeJson(serde_json::Error),

    #[debug(fmt = "Url Parser Error")]
    UrlParser(url::ParseError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<A> = std::result::Result<A, Error>;
