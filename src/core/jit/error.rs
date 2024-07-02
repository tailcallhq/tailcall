use derive_more::From;
use serde::Serialize;

#[derive(From, Debug, Clone)]
pub enum Error {
    #[from(ignore)]
    BuildError(String),
    ParseError(async_graphql::parser::Error),
    IR(crate::core::ir::Error),
}

pub type Result<A> = std::result::Result<A, Error>;

impl Serialize for Error {
    // TODO: this needs a review
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Error::BuildError(msg) => serializer.serialize_str(msg),
            Error::ParseError(err) => serializer.serialize_str(&err.to_string()),
            Error::IR(err) => serializer.serialize_str(&err.to_string()),
        }
    }
}
