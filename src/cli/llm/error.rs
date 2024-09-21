use derive_more::From;
use strum_macros::Display;

#[derive(Debug, From, Display, thiserror::Error)]
pub enum Error {
    GenAI(genai::Error),
    EmptyResponse,
    Serde(serde_json::Error),
    Err(String),
}

pub type Result<A> = std::result::Result<A, Error>;
