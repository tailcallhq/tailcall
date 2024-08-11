use std::string::FromUtf8Error;

use derive_more::From;

use super::file;
use crate::core::config::UnsupportedConfigFormat;
use crate::core::worker;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Worker Error: {}", _0)]
    Worker(worker::Error),

    #[error("File Error: {}", _0)]
    File(file::Error),

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

    #[error("Error: {}", _0)]
    Anyhow(anyhow::Error),
}

pub type Result<A, E> = std::result::Result<A, E>;
