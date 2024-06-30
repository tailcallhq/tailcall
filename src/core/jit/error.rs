use derive_more::From;

#[derive(From, Debug, Clone)]
pub enum Error {
    #[from(ignore)]
    BuildError(String),
    ParseError(async_graphql::parser::Error),
    IR(crate::core::ir::Error),
}

pub type Result<A> = std::result::Result<A, Error>;
