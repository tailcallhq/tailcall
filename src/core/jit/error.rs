use derive_more::From;

#[derive(From, Debug, Clone, strum_macros::Display)]
pub enum Error {
    #[from(ignore)]
    BuildError(String),
    ParseError(async_graphql::parser::Error),
    IR(crate::core::ir::Error),
}

pub type Result<A> = std::result::Result<A, Error>;
