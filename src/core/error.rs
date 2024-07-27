use derive_more::From;

use super::worker;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Worker Error")]
    Worker(worker::Error),
}

pub type Result<A, E> = std::result::Result<A, E>;
