use derive_more::From;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Std Fmt Error")]
    StdFmt(std::fmt::Error),

    #[error("Std IO Error")]
    IO(std::io::Error),

    #[error("Failed to resolve filename")]
    FilenameNotResolved,
}

pub type Result<A> = std::result::Result<A, Error>;
