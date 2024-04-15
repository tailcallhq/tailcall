use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};

#[derive(strum_macros::Display)]
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
            .ok_or(anyhow!("No file extension found"))?
            .to_lowercase();
        match ext.as_str() {
            "gql" | "graphql" => Ok(Parser::Gql),
            "yml" | "yaml" => Ok(Parser::Yml),
            "json" => Ok(Parser::Json),
            "md" => Ok(Parser::Md),
            "ts" => Ok(Parser::Ts),
            "js" => Ok(Parser::Js),
            _ => Err(anyhow!("Unsupported file type")),
        }
    }
}

pub fn format_with_prettier<T: AsRef<str>>(code: T, file_ty: Parser) -> Result<String> {
    let mut child = Command::new("prettier")
        .arg("--stdin-filepath")
        .arg(format!("file.{}", file_ty))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(code.as_ref().as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(anyhow!("Prettier formatting failed"))
    }
}

#[cfg(test)]
mod tests {
    use crate::{format_with_prettier, Parser};

    #[test]
    fn test_js() -> anyhow::Result<()> {
        let prettier = format_with_prettier("const x={a:3};", Parser::Js)?;
        insta::assert_snapshot!(prettier);
        Ok(())
    }
}
