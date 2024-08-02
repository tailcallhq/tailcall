use std::string::FromUtf8Error;
use std::sync::Arc;

use derive_more::From;

use super::file;
use crate::core::config::UnsupportedConfigFormat;
use crate::core::{worker, http};

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Worker Error: {}", _0)]
    Worker(worker::Error),

    #[error("File Error: {}", _0)]
    File(file::Error),

    #[error("Http Error: {}", _0)]
    Http(http::Error),

    #[error("Inquire Error: {}", _0)]
    Inquire(String),

    #[error("Serde Yaml Error: {}", _0)]
    SerdeYaml(serde_yaml::Error),

    #[error("Package name is required")]
    PackageNameNotFound,

    #[error("Protox Parse Error: {}", _0)]
    ProtoxParse(protox_parse::ParseError),

    #[error("Unable to extract content of google well-known proto file")]
    GoogleProtoFileContentNotExtracted,

    #[error("Utf8 Error: {}", _0)]
    Utf8(FromUtf8Error),

    #[error("Unsupported Config Format: {}", _0)]
    UnsupportedConfigFormat(UnsupportedConfigFormat),

    #[error("Unsupported File Format: {}", _0)]
    #[from(ignore)]
    UnsupportedFileFormat(String),

    #[error("Serde Json Error: {}", _0)]
    SerdeJson(serde_json::Error),

    #[error("Unable to determine path")]
    PathDeterminationFailed,

    #[error("Schema mismatch Error")]
    SchemaMismatch,

    #[error("{}\n\nCaused by:\n    {}", context, source)]
    Context { source: Arc<Error>, context: String },

    #[error("Error: {}", _0)]
    Anyhow(anyhow::Error),
}

impl Error {
    pub fn with_context(self, context: String) -> Self {
        Error::Context { source: Arc::new(self), context }
    }
}

pub type Result<A, E> = std::result::Result<A, E>;
