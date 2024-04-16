use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};

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
        let config = config()?;
        self.runtime
            .spawn_blocking(move || {
                let mut command = command();
                let mut child = command
                    .arg("-c")
                    .arg(config)
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

fn config() -> Result<String> {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();
    root_dir.push(".prettierrc");
    Ok(root_dir.to_str().context("Unable to find .prettierrc please raise the issue at https://github.com/tailcallhq/tailcall/issues/")?.to_string())
}

fn command() -> Command {
    if cfg!(target_os = "windows") {
        Command::new("prettier.cmd")
    } else {
        Command::new("prettier")
    }
}
