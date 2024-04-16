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

fn get_command() -> Command {
    if cfg!(target_os = "windows") {
        Command::new("prettier.cmd")
    } else {
        Command::new("prettier")
    }
}

pub fn format<T: AsRef<str>>(source: T, parser: Parser) -> Result<String> {
    let mut child = get_command()
        .arg("--stdin-filepath")
        .arg(format!("file.{}", parser))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(source.as_ref().as_bytes())?;
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
    use crate::{format, Parser};

    #[test]
    fn test_js() -> anyhow::Result<()> {
        let prettier = format("const x={a:3};", Parser::Js)?;
        assert_eq!("const x = {a: 3}\n", prettier);
        Ok(())
    }
}
