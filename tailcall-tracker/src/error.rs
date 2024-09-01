use std::fmt::Display;

use derive_more::{DebugCustom, From};
use reqwest::header::InvalidHeaderValue;

#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Reqwest Error: {}", _0)]
    Reqwest(reqwest::Error),

    #[debug(fmt = "Invalid Header Value: {}", _0)]
    InvalidHeaderValue(InvalidHeaderValue),

    #[debug(fmt = "Serde JSON Error: {}", _0)]
    SerdeJson(serde_json::Error),

    #[debug(fmt = "Url Parser Error: {}", _0)]
    UrlParser(url::ParseError),

    #[debug(fmt = "PostHog Error: {}", _0)]
    PostHog(posthog_rs::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Reqwest(error) => write!(f, "Reqwest Error: {}", error),
            Error::InvalidHeaderValue(error) => write!(f, "Invalid Header Value: {}", error),
            Error::SerdeJson(error) => write!(f, "Serde JSON Error: {}", error),
            Error::UrlParser(error) => write!(f, "Url Parser Error: {}", error),
            Error::PostHog(error) => write!(f, "PostHog Error: {}", error),
        }
    }
}

pub type Result<A> = std::result::Result<A, Error>;
