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

    #[debug(fmt = "Tokio Join Error: {}", _0)]
    TokioJoin(tokio::task::JoinError),
}

pub type Result<A> = std::result::Result<A, Error>;
