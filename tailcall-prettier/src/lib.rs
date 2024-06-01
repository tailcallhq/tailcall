use std::sync::Arc;
mod parser;
mod prettier;
use anyhow::Result;
pub use parser::Parser;
use prettier::Prettier;

lazy_static::lazy_static! {
    static ref PRETTIER: Arc<Prettier> = Arc::new(Prettier::new());
}

pub async fn format<T: AsRef<str>>(source: T, parser: &Parser) -> Result<String> {
    PRETTIER.format(source.as_ref().to_string(), parser).await
}

#[cfg(test)]
mod tests {
    use crate::{format, Parser};

    #[tokio::test]
    async fn test_js() -> anyhow::Result<()> {
        let prettier = format("const x={a:3};", &Parser::Js).await?;
        assert_eq!("const x = {a: 3}\n", prettier);
        Ok(())
    }
}
