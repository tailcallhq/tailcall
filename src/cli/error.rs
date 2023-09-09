use std::fmt::{Debug, Display};

use derive_setters::Setters;
use thiserror::Error;

use crate::blueprint::BlueprintGenerationError;
use crate::valid::ValidationError;

#[derive(Error, Setters)]
pub struct CLIError {
    is_root: bool,
    message: String,
    #[setters(strip_option)]
    description: Option<String>,
    trace: Vec<String>,

    #[setters(skip)]
    caused_by: Box<Vec<CLIError>>,
}

impl CLIError {
    pub fn new(message: &str) -> Self {
        CLIError {
            is_root: true,
            message: message.to_string(),
            description: Default::default(),
            trace: Default::default(),
            caused_by: Default::default(),
        }
    }

    pub fn caused_by(mut self, error: Vec<CLIError>) -> Self {
        self.caused_by = Box::new(error);

        for error in self.caused_by.iter_mut() {
            error.is_root = false;
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
        let error_prefix = "error: ";
        let default_padding = 4;
        let root_padding_size = if self.is_root {
            error_prefix.len()
        } else {
            default_padding
        };
        if self.is_root {
            f.write_str(error_prefix)?;
        }

        f.write_str(&format!("{}", &self.message))?;

        if let Some(description) = &self.description {
            f.write_str("\n")?;
            f.write_str(margin(description, root_padding_size).as_str())?;
        }

        if !self.trace.is_empty() {
            f.write_str(" [at ")?;
            let len = self.trace.len();
            for (i, trace) in self.trace.iter().enumerate() {
                f.write_str(&format!("{}", trace))?;
                if i < len - 1 {
                    f.write_str(".")?;
                }
            }
            f.write_str("]")?;
        }

        if !self.caused_by.is_empty() {
            f.write_str("\nCaused by:\n")?;
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

impl Debug for CLIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl From<BlueprintGenerationError> for CLIError {
    fn from(_error: BlueprintGenerationError) -> Self {
        todo!()
    }
}

impl From<hyper::Error> for CLIError {
    fn from(_error: hyper::Error) -> Self {
        todo!()
    }
}

impl From<ValidationError<String>> for CLIError {
    fn from(_error: ValidationError<String>) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use stripmargin::StripMargin;

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
        let expected = r#"error: Server could not be started"#.strip_margin();
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_title_description() {
        let error = CLIError::new("Server could not be started").description("The port is already in use".to_string());
        let expected = r#"|error: Server could not be started
                          |       The port is already in use"#
            .strip_margin();

        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_title_description_trace() {
        let error = CLIError::new("Server could not be started")
            .description("The port is already in use".to_string())
            .trace(vec!["@server".into(), "port".into()]);

        let expected = r#"|error: Server could not be started
                          |       The port is already in use [at @server.port]"#
            .strip_margin();

        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_title_trace_caused_by() {
        let error =
            CLIError::new("Configuration Error").caused_by(vec![CLIError::new("Base URL needs to be specified")
                .trace(vec!["User".into(), "posts".into(), "@http".into(), "baseURL".into()])]);

        let expected = r#"|error: Configuration Error
                          |Caused by:
                          |    • Base URL needs to be specified [at User.posts.@http.baseURL]"#
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
                .trace(vec!["Query".into(), "users".into(), "@http".into(), "baseURL".into()]),
            CLIError::new("Base URL needs to be specified").trace(vec![
                "Query".into(),
                "posts".into(),
                "@http".into(),
                "baseURL".into(),
            ]),
        ]);

        let expected = r#"|error: Configuration Error
                          |Caused by:
                          |    • Base URL needs to be specified [at User.posts.@http.baseURL]
                          |    • Base URL needs to be specified [at Post.users.@http.baseURL]
                          |    • Base URL needs to be specified
                          |          Set `baseURL` in @http or @server directives [at Query.users.@http.baseURL]
                          |    • Base URL needs to be specified [at Query.posts.@http.baseURL]"#
            .strip_margin();

        assert_eq!(error.to_string(), expected);
    }
}
