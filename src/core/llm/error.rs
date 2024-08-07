use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    GenAI(genai::Error),
    EmptyResponse,
    MissingMarker(String),
    Serde(serde_json::Error),
}

pub type Result<A> = std::result::Result<A, Error>;
