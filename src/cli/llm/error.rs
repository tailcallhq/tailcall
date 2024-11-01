use derive_more::From;
use miette::Diagnostic;
use strum_macros::Display;

#[derive(Debug, From, Display, thiserror::Error, Diagnostic)]
pub enum Error {
    GenAI(genai::Error),
    EmptyResponse,
    Serde(serde_json::Error),
}

pub type Result<A> = std::result::Result<A, Error>;
