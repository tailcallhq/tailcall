use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebcError {
    #[error("Response failed with status {status}: {body}")]
    ResponseFailedStatus { status: StatusCode, body: String },
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("GenAI error: {0}")]
    GenAI(genai::Error),
    #[error("Webc error: {0}")]
    Webc(WebcError),
    #[error("Empty response")]
    EmptyResponse,
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl From<genai::Error> for Error {
    fn from(err: genai::Error) -> Self {
        if let genai::Error::WebModelCall { webc_error, .. } = &err {
            let error_str = webc_error.to_string();
            if error_str.contains("ResponseFailedStatus") {
                // Extract status and body from the error message
                let parts: Vec<&str> = error_str.splitn(3, ": ").collect();
                if parts.len() >= 3 {
                    if let Ok(status) = parts[1].parse::<u16>() {
                        return Error::Webc(WebcError::ResponseFailedStatus {
                            status: StatusCode::from_u16(status)
                                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                            body: parts[2].to_string(),
                        });
                    }
                }
            }
        }
        Error::GenAI(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
