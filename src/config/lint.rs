use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema)]
pub enum TextCase {
    Camel,
    Pascal,
    Snake,
    ScreamingSnake,
}

impl Display for TextCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TextCase::Camel => "camelCase",
            TextCase::Pascal => "PascalCase",
            TextCase::Snake => "snake_case",
            TextCase::ScreamingSnake => "SCREAMING_SNAKE_CASE",
        })
    }
}

/// The @lint directive allows you to configure linting.
#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema)]
pub struct Lint {
    ///
    /// To autoFix the lint.
    /// Example Usage lint:{autoFix:true}
    #[serde(rename = "autoFix")]
    pub auto_fix: Option<bool>,
    ///
    ///
    /// This enum is provided with appropriate TextCase.
    /// Example Usage: lint:{enum:Pascal}
    #[serde(rename = "enum")]
    pub enum_lint: Option<TextCase>,
    ///
    ///
    /// This enumValue is provided with appropriate TextCase.
    /// Example Usage: lint:{enumValue:ScreamingSnake}
    #[serde(rename = "enumValue")]
    pub enum_value_lint: Option<TextCase>,
    ///
    ///
    /// This field is provided with appropriate TextCase.
    /// Example Usage: lint:{field:Camel}
    #[serde(rename = "field")]
    pub field_lint: Option<TextCase>,
    ///
    ///
    /// This type is provided with appropriate TextCase.
    /// Example Usage: lint:{type:Pascal}
    #[serde(rename = "type")]
    pub type_lint: Option<TextCase>,
}
