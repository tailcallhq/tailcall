use std::fmt::{Debug, Display};

use colored::Colorize;
use derive_setters::Setters;
use thiserror::Error;

use crate::valid::ValidationError;

#[derive(Debug, Error, Setters)]
pub struct CLIError {
    is_root: bool,
    #[setters(skip)]
    color: bool,
    message: String,
    #[setters(strip_option)]
    description: Option<String>,
    trace: Vec<String>,

    #[setters(skip)]
    caused_by: Vec<CLIError>,
}

impl CLIError {
    pub fn new(message: &str) -> Self {
        CLIError {
            is_root: true,
            color: false,
            message: message.to_string(),
            description: Default::default(),
            trace: Default::default(),
            caused_by: Default::default(),
        }
    }

    pub fn caused_by(mut self, error: Vec<CLIError>) -> Self {
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

impl Display for CLIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_prefix = "[ERROR] ";
        let default_padding = 2;

        if self.is_root {
            f.write_str(self.colored(error_prefix, colored::Color::Red).as_str())?;
        }

        f.write_str(&self.message.to_string())?;

        if let Some(description) = &self.description {
            f.write_str(
                &self.colored(format!(": {}", description).as_str(), colored::Color::White),
            )?;
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
            f.write_str(self.colored(error_prefix, colored::Color::Red).as_str())?;
            f.write_str(self.dimmed("Caused by:\n").as_str())?;
            for (i, error) in self.caused_by.iter().enumerate() {
                let message = &error.to_string();
                f.write_str(self.colored(error_prefix, colored::Color::Red).as_str())?;

                f.write_str(&margin(bullet(message.as_str()).as_str(), default_padding))?;

                if i < self.caused_by.len() - 1 {
                    f.write_str("\n")?;
                }
            }
        }

        Ok(())
    }
}

impl From<hyper::Error> for CLIError {
    fn from(error: hyper::Error) -> Self {
        // TODO: add type-safety to CLIError conversion
        let cli_error = CLIError::new("Server Failed");
        let message = error.to_string();
        if message.to_lowercase().contains("os error 48") {
            cli_error
                .description("The port is already in use".to_string())
                .caused_by(vec![CLIError::new(message.as_str())])
        } else {
            cli_error.description(message)
        }
    }
}

impl From<rustls::Error> for CLIError {
    fn from(error: rustls::Error) -> Self {
        let cli_error = CLIError::new("Failed to create TLS Acceptor");
        let message = error.to_string();

        cli_error.description(message)
    }
}

impl From<std::io::Error> for CLIError {
    fn from(error: std::io::Error) -> Self {
        let cli_error = CLIError::new("IO Error");
        let message = error.to_string();

        cli_error.description(message)
    }
}

impl<'a> From<ValidationError<&'a str>> for CLIError {
    fn from(error: ValidationError<&'a str>) -> Self {
        CLIError::new("Invalid Configuration").caused_by(
            error
                .as_vec()
                .iter()
                .map(|cause| {
                    let mut err =
                        CLIError::new(cause.message).trace(Vec::from(cause.trace.clone()));
                    if let Some(description) = cause.description {
                        err = err.description(description.to_owned());
                    }
                    err
                })
                .collect(),
        )
    }
}

impl From<ValidationError<String>> for CLIError {
    fn from(error: ValidationError<String>) -> Self {
        CLIError::new("Invalid Configuration").caused_by(
            error
                .as_vec()
                .iter()
                .map(|cause| {
                    CLIError::new(cause.message.as_str()).trace(Vec::from(cause.trace.clone()))
                })
                .collect(),
        )
    }
}

impl From<Box<dyn std::error::Error>> for CLIError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        CLIError::new(value.to_string().as_str())
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use stripmargin::StripMargin;

    use super::*;
    use crate::valid::Cause;

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
        let error = CLIError::new("Server could not be started");
        let expected = r"[ERROR] Server could not be started".strip_margin();
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_title_description() {
        let error = CLIError::new("Server could not be started")
            .description("The port is already in use".to_string());
        let expected = r"|[ERROR] Server could not be started: The port is already in use"
            .strip_margin();

        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_title_description_trace() {
        let error = CLIError::new("Server could not be started")
            .description("The port is already in use".to_string())
            .trace(vec!["@server".into(), "port".into()]);

        let expected = r"|[ERROR] Server could not be started: The port is already in use [at @server.port]"
            .strip_margin();

        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_title_trace_caused_by() {
        let error = CLIError::new("Configuration Error").caused_by(vec![CLIError::new(
            "Base URL needs to be specified",
        )
        .trace(vec![
            "User".into(),
            "posts".into(),
            "@http".into(),
            "baseURL".into(),
        ])]);

        let expected = r"|[ERROR] Configuration Error
                     |[ERROR] Caused by:
                     |[ERROR]   • Base URL needs to be specified [at User.posts.@http.baseURL]"
            .strip_margin();

        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_title_trace_multiple_caused_by() {
        let error = CLIError::new("Configuration Error").caused_by(vec![
            CLIError::new("Base URL needs to be specified").trace(vec![
                "User".into(),
                "posts".into(),
                "@http".into(),
                "baseURL".into(),
            ]),
            CLIError::new("Base URL needs to be specified").trace(vec![
                "Post".into(),
                "users".into(),
                "@http".into(),
                "baseURL".into(),
            ]),
            CLIError::new("Base URL needs to be specified")
                .description("Set `baseURL` in @http or @server directives".into())
                .trace(vec![
                    "Query".into(),
                    "users".into(),
                    "@http".into(),
                    "baseURL".into(),
                ]),
            CLIError::new("Base URL needs to be specified").trace(vec![
                "Query".into(),
                "posts".into(),
                "@http".into(),
                "baseURL".into(),
            ]),
        ]);

        let expected = r"|[ERROR] Configuration Error
                     |[ERROR] Caused by:
                     |[ERROR]   • Base URL needs to be specified [at User.posts.@http.baseURL]
                     |[ERROR]   • Base URL needs to be specified [at Post.users.@http.baseURL]
                     |[ERROR]   • Base URL needs to be specified: Set `baseURL` in @http or @server directives [at Query.users.@http.baseURL]
                     |[ERROR]   • Base URL needs to be specified [at Query.posts.@http.baseURL]"
            .strip_margin();

        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_from_validation() {
        let cause = Cause::new("Base URL needs to be specified")
            .description("Set `baseURL` in @http or @server directives")
            .trace(vec!["Query", "users", "@http", "baseURL"]);
        let valid = ValidationError::from(cause);
        let error = CLIError::from(valid);
        let expected = r"|[ERROR] Invalid Configuration
                     |[ERROR] Caused by:
                     |[ERROR]   • Base URL needs to be specified: Set `baseURL` in @http or @server directives [at Query.users.@http.baseURL]"
            .strip_margin();

        assert_eq!(error.to_string(), expected);
    }
}
