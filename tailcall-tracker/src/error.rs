use derive_more::{Debug, From};
use reqwest::header::InvalidHeaderValue;

#[derive(From, Debug)]
pub enum Error {
    #[debug("Reqwest Error: {}", _0)]
    Reqwest(reqwest::Error),

    #[debug("Invalid Header Value: {}", _0)]
    InvalidHeaderValue(InvalidHeaderValue),

    #[debug("Serde JSON Error: {}", _0)]
    SerdeJson(serde_json::Error),

    #[debug("Url Parser Error: {}", _0)]
    UrlParser(url::ParseError),

    #[debug("PostHog Error: {}", _0)]
    PostHog(posthog_rs::Error),

    #[debug("Tokio Join Error: {}", _0)]
    TokioJoin(tokio::task::JoinError),

    #[debug("IO Error: {}", _0)]
    IO(std::io::Error),
}

pub type Result<A> = std::result::Result<A, Error>;
