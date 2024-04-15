
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};

pub fn format_with_prettier<T: AsRef<str>>(code: T, file_ty: T) -> Result<String> {
    let mut child = Command::new("prettier")
        .arg("--stdin-filepath")
        .arg(file_ty.as_ref())
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
