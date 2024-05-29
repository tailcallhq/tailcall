use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};
use tokio::runtime::Runtime;

use crate::Parser;

/// Struct representing a Prettier formatter.
///
/// # Fields
///
/// * `runtime` - A Tokio runtime for executing asynchronous tasks.
/// * `config_path` - An optional path to the Prettier configuration file.
pub struct Prettier {
    runtime: Runtime,
    config_path: Option<String>,
}

impl Prettier {
    /// Creates a new `Prettier` instance.
    ///
    /// This function initializes a new multi-threaded Tokio runtime with a
    /// maximum of 1024 blocking threads and attempts to locate a
    /// `.prettierrc` configuration file in the current directory.
    ///
    /// # Returns
    ///
    /// A new `Prettier` instance.
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

    /// Formats the provided source code string using Prettier.
    ///
    /// This method spawns a blocking task on the Tokio runtime to execute the
    /// Prettier command-line tool. It passes the source code to Prettier
    /// via stdin and captures the formatted output from stdout.
    ///
    /// # Arguments
    ///
    /// * `source` - A string containing the source code to be formatted.
    /// * `parser` - A reference to a `Parser` that specifies the language
    ///   parser to be used.
    ///
    /// # Returns
    ///
    /// A `Result` containing the formatted source code string or an error if
    /// formatting fails.
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

/// Returns a `Command` to execute Prettier based on the target operating
/// system.
///
/// On Windows, this function returns a command to execute `prettier.cmd`, while
/// on other operating systems, it returns a command to execute `prettier`.
///
/// # Returns
///
/// A `Command` instance for executing Prettier.
fn command() -> Command {
    if cfg!(target_os = "windows") {
        Command::new("prettier.cmd")
    } else {
        Command::new("prettier")
    }
}
