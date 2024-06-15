use derive_more::From;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Worker Error")]
    Worker(worker::Error),

    #[error("File {0} was not found in bucket")]
    MissingFileInBucket(String),
}

pub type Result<A> = std::result::Result<A, Error>;
