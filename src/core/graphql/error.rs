use derive_more::From;

use crate::core::http;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Serde Json Error: {}", _0)]
    SerdeJson(serde_json::Error),

    #[error("HTTP Error: {}", _0)]
    Http(http::Error),
}