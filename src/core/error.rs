use std::fmt::{Debug, Display};

use colored::Colorize;
use derive_setters::Setters;

use crate::core::valid::ValidationError;

#[derive(Debug, thiserror::Error, Setters, PartialEq, Clone)]
pub struct Error {
    is_root: bool,
    #[setters(skip)]
    color: bool,
    message: String,
    #[setters(strip_option)]
    description: Option<String>,
    trace: Vec<String>,

    #[setters(skip)]
    caused_by: Vec<Error>,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error {
            is_root: true,
            color: false,
            message: message.to_string(),
            description: Default::default(),
            trace: Default::default(),
            caused_by: Default::default(),
        }
    }

    pub fn caused_by(mut self, error: Vec<Error>) -> Self {
        self.caused_by = error;

        for error in self.caused_by.iter_mut() {
            error.is_root = false;
        }

        self
    }

    fn colored<'a>(&'a self, str: &'a str, color: colored::Color) -> String {
        if self.color {
            str.color(color).to_string()
        } else {
            str.to_string()
        }
    }

    fn dimmed<'a>(&'a self, str: &'a str) -> String {
        if self.color {
            str.dimmed().to_string()
        } else {
            str.to_string()
        }
    }

    pub fn color(mut self, color: bool) -> Self {
        self.color = color;
        for inner in self.caused_by.iter_mut() {
            inner.color = color;
        }
        self
    }
}

fn margin(str: &str, margin: usize) -> String {
    let mut result = String::new();
    for line in str.split_inclusive('\n') {
        result.push_str(&format!("{}{}", " ".repeat(margin), line));
    }
    result
}

fn bullet(str: &str) -> String {
    let mut chars = margin(str, 2).chars().collect::<Vec<char>>();
    chars[0] = '•';
    chars[1] = ' ';
    chars.into_iter().collect::<String>()
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let default_padding = 2;

        let message_color = if self.is_root {
            colored::Color::Yellow
        } else {
            colored::Color::White
        };

        f.write_str(self.colored(&self.message, message_color).as_str())?;

        if let Some(description) = &self.description {
            f.write_str(&self.colored(": ", message_color))?;
            f.write_str(&self.colored(description.to_string().as_str(), colored::Color::White))?;
        }

        if !self.trace.is_empty() {
            let mut buf = String::new();
            buf.push_str(" [at ");
            let len = self.trace.len();
            for (i, trace) in self.trace.iter().enumerate() {
                buf.push_str(&trace.to_string());
                if i < len - 1 {
                    buf.push('.');
                }
            }
            buf.push(']');
            f.write_str(&self.colored(&buf, colored::Color::Cyan))?;
        }

        if !self.caused_by.is_empty() {
            f.write_str("\n")?;
            f.write_str(self.dimmed("Caused by:").as_str())?;
            f.write_str("\n")?;
            for (i, error) in self.caused_by.iter().enumerate() {
                let message = &error.to_string();

                f.write_str(&margin(bullet(message.as_str()).as_str(), default_padding))?;

                if i < self.caused_by.len() - 1 {
                    f.write_str("\n")?;
                }
            }
        }

        Ok(())
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Self {
        // TODO: add type-safety to CLIError conversion
        let cli_error = Error::new("Server Failed");
        let message = error.to_string();
        if message.to_lowercase().contains("os error 48") {
            cli_error
                .description("The port is already in use".to_string())
                .caused_by(vec![Error::new(message.as_str())])
        } else {
            cli_error.description(message)
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        // Convert other errors to CLIError
        let cli_error = match error.downcast::<Error>() {
            Ok(cli_error) => cli_error,
            Err(error) => {
                // Convert other errors to CLIError
                let cli_error = match error.downcast::<ValidationError<String>>() {
                    Ok(validation_error) => Error::from(validation_error),
                    Err(error) => {
                        let sources = error
                            .source()
                            .map(|error| vec![Error::new(error.to_string().as_str())])
                            .unwrap_or_default();

                        Error::new(&error.to_string()).caused_by(sources)
                    }
                };
                cli_error
            }
        };
        cli_error
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        let cli_error = Error::new("IO Error");
        let message = error.to_string();

        cli_error.description(message)
    }
}

impl<'a> From<ValidationError<&'a str>> for Error {
    fn from(error: ValidationError<&'a str>) -> Self {
        Error::new("Invalid Configuration").caused_by(
            error
                .as_vec()
                .iter()
                .map(|cause| {
                    let mut err = Error::new(cause.message).trace(Vec::from(cause.trace.clone()));
                    if let Some(description) = cause.description {
                        err = err.description(description.to_owned());
                    }
                    err
                })
                .collect(),
        )
    }
}

impl From<ValidationError<String>> for Error {
    fn from(error: ValidationError<String>) -> Self {
        Error::new("Invalid Configuration").caused_by(
            error
                .as_vec()
                .iter()
                .map(|cause| {
                    Error::new(cause.message.as_str()).trace(Vec::from(cause.trace.clone()))
                })
                .collect(),
        )
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Error::new(value.to_string().as_str())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::valid::Cause;

    #[test]
    fn test_no_newline() {
        let input = "Hello";
        let expected = "    Hello";
        assert_eq!(margin(input, 4), expected);
    }

    #[test]
    fn test_with_newline() {
        let input = "Hello\nWorld";
        let expected = "    Hello\n    World";
        assert_eq!(margin(input, 4), expected);
    }

    #[test]
    fn test_empty_string() {
        let input = "";
        let expected = "";
        assert_eq!(margin(input, 4), expected);
    }

    #[test]
    fn test_zero_margin() {
        let input = "Hello";
        let expected = "Hello";
        assert_eq!(margin(input, 0), expected);
    }

    #[test]
    fn test_zero_margin_with_newline() {
        let input = "Hello\nWorld";
        let expected = "Hello\nWorld";
        assert_eq!(margin(input, 0), expected);
    }

    #[test]
    fn test_title() {
        let error = Error::new("Server could not be started");
        insta::assert_snapshot!(error.to_string());
    }

    #[test]
    fn test_title_description() {
        let error = Error::new("Server could not be started")
            .description("The port is already in use".to_string());

        insta::assert_snapshot!(error.to_string());
    }

    #[test]
    fn test_title_description_trace() {
        let error = Error::new("Server could not be started")
            .description("The port is already in use".to_string())
            .trace(vec!["@server".into(), "port".into()]);

        insta::assert_snapshot!(error.to_string());
    }

    #[test]
    fn test_title_trace_caused_by() {
        let error = Error::new("Configuration Error").caused_by(vec![Error::new(
            "Base URL needs to be specified",
        )
        .trace(vec![
            "User".into(),
            "posts".into(),
            "@http".into(),
            "baseURL".into(),
        ])]);

        insta::assert_snapshot!(error.to_string());
    }

    #[test]
    fn test_title_trace_multiple_caused_by() {
        let error = Error::new("Configuration Error").caused_by(vec![
            Error::new("Base URL needs to be specified").trace(vec![
                "User".into(),
                "posts".into(),
                "@http".into(),
                "baseURL".into(),
            ]),
            Error::new("Base URL needs to be specified").trace(vec![
                "Post".into(),
                "users".into(),
                "@http".into(),
                "baseURL".into(),
            ]),
            Error::new("Base URL needs to be specified")
                .description("Set `baseURL` in @http or @server directives".into())
                .trace(vec![
                    "Query".into(),
                    "users".into(),
                    "@http".into(),
                    "baseURL".into(),
                ]),
            Error::new("Base URL needs to be specified").trace(vec![
                "Query".into(),
                "posts".into(),
                "@http".into(),
                "baseURL".into(),
            ]),
        ]);

        insta::assert_snapshot!(error.to_string());
    }

    #[test]
    fn test_from_validation() {
        let cause = Cause::new("Base URL needs to be specified")
            .description("Set `baseURL` in @http or @server directives")
            .trace(vec!["Query", "users", "@http", "baseURL"]);
        let valid = ValidationError::from(cause);
        let error = Error::from(valid);

        insta::assert_snapshot!(error.to_string());
    }

    #[test]
    fn test_cli_error_identity() {
        let cli_error = Error::new("Server could not be started")
            .description("The port is already in use".to_string())
            .trace(vec!["@server".into(), "port".into()]);
        let anyhow_error: anyhow::Error = cli_error.clone().into();

        let actual = Error::from(anyhow_error);
        let expected = cli_error;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_validation_error_identity() {
        let validation_error = ValidationError::from(
            Cause::new("Test Error".to_string()).trace(vec!["Query".to_string()]),
        );
        let anyhow_error: anyhow::Error = validation_error.clone().into();

        let actual = Error::from(anyhow_error);
        let expected = Error::from(validation_error);

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_generic_error() {
        let anyhow_error = anyhow::anyhow!("Some error msg");

        let actual: Error = Error::from(anyhow_error);
        let expected = Error::new("Some error msg");

        assert_eq!(actual, expected);
    }
}
