use std::error::Error;
use std::fmt::{Debug, Display};

use colored::Colorize;
use thiserror::Error;

use crate::blueprint::BlueprintGenerationError;

#[derive(Error)]
pub enum CLIError {
    BlueprintGenerationError(BlueprintGenerationError),
    LaunchError(hyper::Error),
}

fn write_err(f: &mut std::fmt::Formatter<'_>, err: String) -> std::fmt::Result {
    f.write_str(&err.bright_red().bold())
}

impl Display for CLIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CLIError::BlueprintGenerationError(BlueprintGenerationError(error)) => {
                f.write_str("Invalid Configuration\n")?;

                for error in error.as_vec() {
                    let trace = format!(" [{}]", error.trace.iter().cloned().collect::<Vec<String>>().join(", "))
                        .dimmed()
                        .to_string();

                    write_err(f, format!("{} {}", '\u{2022}', error.message))?;
                    f.write_str(&trace)?;
                    f.write_str("\n")?;
                }
            }
            CLIError::LaunchError(error) => {
                if error.source().is_some() {
                    write_err(f, format!("{}", error.source().unwrap()))?;
                } else {
                    write_err(f, format!("{}", error))?;
                }
            }
        };

        Ok(())
    }
}

impl Debug for CLIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl From<BlueprintGenerationError> for CLIError {
    fn from(error: BlueprintGenerationError) -> Self {
        CLIError::BlueprintGenerationError(error)
    }
}

impl From<hyper::Error> for CLIError {
    fn from(error: hyper::Error) -> Self {
        CLIError::LaunchError(error)
    }
}
