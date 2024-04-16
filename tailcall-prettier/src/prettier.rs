use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};

pub use super::Parser;

pub struct Prettier {
    runtime: tokio::runtime::Runtime,
}

impl Prettier {
    pub fn new() -> Prettier {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .max_blocking_threads(1024)
            .build()
            .unwrap();

        Self { runtime }
    }

    pub async fn format(&self, source: String, parser: Parser) -> Result<String> {
        self.runtime
            .spawn_blocking(move || {
                let mut command = command();
                let mut child = command
                    .arg("--stdin-filepath")
                    .arg(format!("file.{}", parser))
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?;

                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(source.as_bytes())?;
                }

                let output = child.wait_with_output()?;
                if output.status.success() {
                    Ok(String::from_utf8(output.stdout)?)
                } else {
                    Err(anyhow!("Prettier formatting failed"))
                }
            })
            .await?
    }
}

fn command() -> Command {
    if cfg!(target_os = "windows") {
        Command::new("prettier.cmd")
    } else {
        Command::new("prettier")
    }
}
