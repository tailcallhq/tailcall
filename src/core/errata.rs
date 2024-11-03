use std::fmt::{Debug, Display};

use colored::Colorize;
use derive_setters::Setters;
use tailcall_valid::ValidationError;

use crate::core::error::Error as CoreError;

/// The moral equivalent of a serde_json::Value but for errors.
/// It's a data structure like Value that can hold any error in an untyped
/// manner.
#[derive(Debug, thiserror::Error, Setters, PartialEq, Clone)]
pub struct Errata {
    is_root: bool,
    #[setters(skip)]
    color: bool,
    message: String,
    #[setters(strip_option)]
    description: Option<String>,
    trace: Vec<String>,

    #[setters(skip)]
    caused_by: Vec<Errata>,
}

impl Errata {
    pub fn new(message: &str) -> Self {
        Errata {
            is_root: true,
            color: false,
            message: message.to_string(),
            description: Default::default(),
            trace: Default::default(),
            caused_by: Default::default(),
        }
    }

    pub fn caused_by(mut self, error: Vec<Errata>) -> Self {
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

impl Display for Errata {
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

impl From<hyper::Error> for Errata {
    fn from(error: hyper::Error) -> Self {
        // TODO: add type-safety to Errata conversion
        let cli_error = Errata::new("Server Failed");
        let message = error.to_string();
        if message.to_lowercase().contains("os error 48") {
            cli_error
                .description("The port is already in use".to_string())
                .caused_by(vec![Errata::new(message.as_str())])
        } else {
            cli_error.description(message)
        }
    }
}

impl From<anyhow::Error> for Errata {
    fn from(error: anyhow::Error) -> Self {
        // Convert other errors to Errata
        let cli_error = match error.downcast::<Errata>() {
            Ok(cli_error) => cli_error,
            Err(error) => {
                // Convert other errors to Errata
                let cli_error = match error.downcast::<ValidationError<String>>() {
                    Ok(validation_error) => Errata::from(validation_error),
                    Err(error) => {
                        let sources = error
                            .source()
                            .map(|error| vec![Errata::new(error.to_string().as_str())])
                            .unwrap_or_default();

                        Errata::new(&error.to_string()).caused_by(sources)
                    }
                };
                cli_error
            }
        };
        cli_error
    }
}

impl From<std::io::Error> for Errata {
    fn from(error: std::io::Error) -> Self {
        let cli_error = Errata::new("IO Error");
        let message = error.to_string();

        cli_error.description(message)
    }
}

impl From<CoreError> for Errata {
    fn from(error: CoreError) -> Self {
        let cli_error = Errata::new("Core Error");
        let message = error.to_string();

        cli_error.description(message)
    }
}

impl<'a> From<ValidationError<&'a str>> for Errata {
    fn from(error: ValidationError<&'a str>) -> Self {
        Errata::new("Invalid Configuration").caused_by(
            error
                .as_vec()
                .iter()
                .map(|cause| {
                    let mut err = Errata::new(cause.message).trace(Vec::from(cause.trace.clone()));
                    if let Some(description) = cause.description {
                        err = err.description(description.to_owned());
                    }
                    err
                })
                .collect(),
        )
    }
}

impl From<ValidationError<String>> for Errata {
    fn from(error: ValidationError<String>) -> Self {
        Errata::new("Invalid Configuration").caused_by(
            error
                .as_vec()
                .iter()
                .map(|cause| {
                    Errata::new(cause.message.as_str()).trace(Vec::from(cause.trace.clone()))
                })
                .collect(),
        )
    }
}

impl From<Box<dyn std::error::Error>> for Errata {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Errata::new(value.to_string().as_str())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use stripmargin::StripMargin;
    use tailcall_valid::Cause;

    use super::*;

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
        let error = Errata::new("Server could not be started");
        let expected = r"Server could not be started".strip_margin();
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_title_description() {
        let error = Errata::new("Server could not be started")
            .description("The port is already in use".to_string());
        let expected = r"|Server could not be started: The port is already in use".strip_margin();

        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_title_description_trace() {
        let error = Errata::new("Server could not be started")
            .description("The port is already in use".to_string())
            .trace(vec!["@server".into(), "port".into()]);

        let expected =
            r"|Server could not be started: The port is already in use [at @server.port]"
                .strip_margin();

        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_title_trace_caused_by() {
        let error = Errata::new("Configuration Error").caused_by(vec![Errata::new(
            "URL needs to be specified",
        )
        .trace(vec![
            "User".into(),
            "posts".into(),
            "@http".into(),
            "url".into(),
        ])]);

        let expected = r"|Configuration Error
                     |Caused by:
                     |  • URL needs to be specified [at User.posts.@http.url]"
            .strip_margin();

        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_title_trace_multiple_caused_by() {
        let error = Errata::new("Configuration Error").caused_by(vec![
            Errata::new("URL needs to be specified").trace(vec![
                "User".into(),
                "posts".into(),
                "@http".into(),
                "url".into(),
            ]),
            Errata::new("URL needs to be specified").trace(vec![
                "Post".into(),
                "users".into(),
                "@http".into(),
                "url".into(),
            ]),
            Errata::new("URL needs to be specified")
                .description("Set `url` in @http or @server directives".into())
                .trace(vec![
                    "Query".into(),
                    "users".into(),
                    "@http".into(),
                    "url".into(),
                ]),
            Errata::new("URL needs to be specified").trace(vec![
                "Query".into(),
                "posts".into(),
                "@http".into(),
                "url".into(),
            ]),
        ]);

        let expected = r"|Configuration Error
                     |Caused by:
                     |  • URL needs to be specified [at User.posts.@http.url]
                     |  • URL needs to be specified [at Post.users.@http.url]
                     |  • URL needs to be specified: Set `url` in @http or @server directives [at Query.users.@http.url]
                     |  • URL needs to be specified [at Query.posts.@http.url]"
            .strip_margin();

        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_from_validation() {
        let cause = Cause::new("URL needs to be specified")
            .description("Set `url` in @http or @server directives")
            .trace(vec!["Query", "users", "@http", "url"]);
        let valid = ValidationError::from(cause);
        let error = Errata::from(valid);
        let expected = r"|Invalid Configuration
                     |Caused by:
                     |  • URL needs to be specified: Set `url` in @http or @server directives [at Query.users.@http.url]"
            .strip_margin();

        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_cli_error_identity() {
        let cli_error = Errata::new("Server could not be started")
            .description("The port is already in use".to_string())
            .trace(vec!["@server".into(), "port".into()]);
        let anyhow_error: anyhow::Error = cli_error.clone().into();

        let actual = Errata::from(anyhow_error);
        let expected = cli_error;

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_validation_error_identity() {
        let validation_error = ValidationError::from(
            Cause::new("Test Error".to_string()).trace(vec!["Query".to_string()]),
        );
        let anyhow_error: anyhow::Error = validation_error.clone().into();

        let actual = Errata::from(anyhow_error);
        let expected = Errata::from(validation_error);

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_generic_error() {
        let anyhow_error = anyhow::anyhow!("Some error msg");

        let actual: Errata = Errata::from(anyhow_error);
        let expected = Errata::new("Some error msg");

        assert_eq!(actual, expected);
    }
}
