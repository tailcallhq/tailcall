use derive_more::From;
use reqwest::header::InvalidHeaderValue;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Reqwest Error")]
    Reqwest(reqwest::Error),

    #[error("Invalid Header Value")]
    InvalidHeaderValue(InvalidHeaderValue),

    #[error("Serde JSON Error")]
    SerdeJson(serde_json::Error),

    #[error("Url Parser Error")]
    UrlParser(url::ParseError),
}

pub type Result<A> = std::result::Result<A, Error>;
