use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};

pub use super::Parser;

pub struct Prettier {
    runtime: tokio::runtime::Runtime,
    config_path: Option<String>,
}

impl Prettier {
    pub fn new() -> Prettier {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .max_blocking_threads(1024)
            .build()
            .unwrap();

        let config_path = fs::canonicalize(Path::new("./.prettierrc"))
            .map(|a| a.display().to_string())
            .ok();
        Self { runtime, config_path }
    }

    pub async fn format<'a>(&'a self, source: String, parser: &'a Parser) -> Result<String> {
        let parser = parser.clone();
        let config = self.config_path.clone();
        self.runtime
            .spawn_blocking(move || {
                let mut command = command();
                let mut child = command
                    .arg("--stdin-filepath")
                    .arg(format!("file.{}", parser));

                if let Some(config) = config {
                    child = child.arg("--config").arg(config);
                }

                let mut child = child.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(source.as_bytes())?;
                }

                let output = child.wait_with_output()?;
                if output.status.success() {
                    Ok(String::from_utf8(output.stdout)?)
                } else {
                    Err(anyhow!(
                        "Prettier formatting failed: {}",
                        String::from_utf8(output.stderr).unwrap()
                    ))
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
