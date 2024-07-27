use super::worker;
use derive_more::From;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Worker Error")]
    Worker(worker::error::Error),
}

pub type Result<A, E> = std::result::Result<A, E>;