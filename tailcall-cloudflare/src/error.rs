use std::string::FromUtf8Error;

use derive_more::From;
use tailcall::core::error;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Worker Error")]
    Worker(worker::Error),

    #[error("File {0} was not found in bucket")]
    MissingFileInBucket(String),

    #[error("BUCKET var is not set")]
    BucketVarNotSet,

    #[error("Hyper Error")]
    Hyper(hyper::Error),

    #[error("FromUtf8 Error")]
    FromUtf8(FromUtf8Error),

    #[error("Unsupported HTTP method: {0}")]
    #[from(ignore)]
    UnsupportedHttpMethod(String),

    #[error("Hyper HTTP Error")]
    HyperHttp(hyper::http::Error),

    #[error("Core Error")]
    Core(error::Error),
}

pub type Result<A> = std::result::Result<A, Error>;
