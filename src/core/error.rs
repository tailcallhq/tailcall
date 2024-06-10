use std::string::FromUtf8Error;

use derive_more::From;

// This is currently not getting used
#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Std IO Error")]
    StdIOError(std::io::Error),

    #[error("Utf8 Error")]
    Utf8Error(FromUtf8Error),
}

pub type Result<A, E> = std::result::Result<A, E>;
