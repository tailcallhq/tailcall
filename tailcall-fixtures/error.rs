use derive_more::{From, DebugCustom};
use std::fmt::Display;


#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Std Fmt Error")]
    StdFmt(std::fmt::Error),

    #[debug(fmt = "Std IO Error")]
    IO(std::io::Error),

    #[debug(fmt = "Failed to resolve filename")]
    FilenameNotResolved,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<A> = std::result::Result<A, Error>;
