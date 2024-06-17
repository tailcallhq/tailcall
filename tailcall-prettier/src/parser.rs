use super::{Error, Result};

#[derive(strum_macros::Display, Clone)]
pub enum Parser {
    Gql,
    Yml,
    Json,
    Md,
    Ts,
    Js,
}

impl Parser {
    pub fn detect(path: &str) -> Result<Self> {
        let ext = path
            .split('.')
            .last()
            .ok_or(Error::FileExtensionNotFound)?
            .to_lowercase();
        match ext.as_str() {
            "gql" | "graphql" => Ok(Parser::Gql),
            "yml" | "yaml" => Ok(Parser::Yml),
            "json" => Ok(Parser::Json),
            "md" => Ok(Parser::Md),
            "ts" => Ok(Parser::Ts),
            "js" => Ok(Parser::Js),
            _ => Err(Error::UnsupportedFiletype),
        }
    }
}
