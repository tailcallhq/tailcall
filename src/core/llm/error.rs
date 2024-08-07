use derive_more::From;

#[derive(From)]
pub enum Error {
    GenAI(genai::Error),
    EmptyResponse,
    MissingMarker(String),
    Serde(serde_json::Error),
}
