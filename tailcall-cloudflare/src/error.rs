use std::string::FromUtf8Error;

use derive_more::{From, DebugCustom};
use tailcall::core::error;
use std::fmt::Display;


#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Worker Error")]
    Worker(worker::Error),

    #[debug(fmt = "File {} was not found in bucket", _0)]
    MissingFileInBucket(String),

    #[debug(fmt = "BUCKET var is not set")]
    BucketVarNotSet,

    #[debug(fmt = "Hyper Error")]
    Hyper(hyper::Error),

    #[debug(fmt = "FromUtf8 Error")]
    FromUtf8(FromUtf8Error),

    #[debug(fmt = "Unsupported HTTP method: {}", _0)]
    #[from(ignore)]
    UnsupportedHttpMethod(String),

    #[debug(fmt = "Hyper HTTP Error")]
    HyperHttp(hyper::http::Error),

    #[debug(fmt = "Core Error")]
    Core(error::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<A> = std::result::Result<A, Error>;
