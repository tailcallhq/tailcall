use std::fmt::{Display, Formatter};
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};

enum FileTypes {
    Gql,
    Yml,
    Json,
    Md,
    Ts,
    Js,
}

impl Display for FileTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileTypes::Gql => write!(f, "graphql"),
            FileTypes::Yml => write!(f, "yml"),
            FileTypes::Json => write!(f, "json"),
            FileTypes::Md => write!(f, "md"),
            FileTypes::Ts => write!(f, "ts"),
            FileTypes::Js => write!(f, "js"),
        }
    }
}

impl FileTypes {
    fn detect(path: &str) -> Result<Self> {
        let ext = path
            .split('.')
            .last()
            .ok_or(anyhow!("No file extension found"))?
            .to_lowercase();
        match ext.as_str() {
            "gql" | "graphql" => Ok(FileTypes::Gql),
            "yml" | "yaml" => Ok(FileTypes::Yml),
            "json" => Ok(FileTypes::Json),
            "md" => Ok(FileTypes::Md),
            "ts" => Ok(FileTypes::Ts),
            "js" => Ok(FileTypes::Js),
            _ => Err(anyhow!("Unsupported file type")),
        }
    }
}

pub fn format_with_prettier<T: AsRef<str>>(code: T, file_ty: T) -> Result<String> {
    let file_ty = FileTypes::detect(file_ty.as_ref())?;

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
    use crate::format_with_prettier;

    #[test]
    fn test_js() -> anyhow::Result<()> {
        let prettier = format_with_prettier("const x={a:3};", "file.ts")?;
        insta::assert_snapshot!(prettier);
        Ok(())
    }
}
