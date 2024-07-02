use std::error::Error as StdError;
use std::fmt::{Display, Formatter};

use derive_more::From;

#[derive(From, Debug, Clone)]
pub enum Error {
    #[from(ignore)]
    BuildError(String),
    ParseError(async_graphql::parser::Error),
    IR(crate::core::ir::Error),
}

pub type Result<A> = std::result::Result<A, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BuildError(e) => write!(f, "BuildError: {}", e),
            Error::ParseError(e) => write!(f, "ParseError: {}", e),
            Error::IR(e) => write!(f, "IR: {}", e),
        }
    }
}

impl StdError for Error {}
