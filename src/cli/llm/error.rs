use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    GenAI(genai::Error),
    EmptyResponse,
    MissingMarker(String),
    Serde(serde_json::Error),
}

impl From<Error> for anyhow::Error {
    fn from(error: Error) -> Self {
        match error {
            Error::GenAI(err) => anyhow::Error::new(err),
            Error::EmptyResponse => anyhow::Error::msg("No response received from the server."),
            Error::MissingMarker(marker) => {
                anyhow::Error::msg(format!("Missing marker: {}", marker))
            }
            Error::Serde(err) => anyhow::Error::new(err),
        }
    }
}

pub type Result<A> = std::result::Result<A, Error>;