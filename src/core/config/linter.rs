use std::collections::BTreeSet;

use anyhow::Result;
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

use super::Config;
use crate::core::config::Variant;
use crate::core::macros::MergeRight;
use crate::core::valid::{Valid, Validator};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema, MergeRight)]
pub enum TextCase {
    #[serde(rename = "camelCase")]
    CamelCase,
    #[serde(rename = "pascalCase")]
    PascalCase,
    #[serde(rename = "snakeCase")]
    SnakeCase,
    #[serde(rename = "screamingSnakeCase")]
    ScreamingSnakeCase,
    #[serde(rename = "allCaps")]
    AllCaps,
}

impl From<TextCase> for Case {
    fn from(text_case: TextCase) -> Self {
        match text_case {
            TextCase::CamelCase => Case::Camel,
            TextCase::PascalCase => Case::Pascal,
            TextCase::SnakeCase => Case::Snake,
            TextCase::ScreamingSnakeCase => Case::ScreamingSnake,
            TextCase::AllCaps => Case::Upper,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema, MergeRight)]
pub struct Linter {
    pub default: Option<bool>,
    #[serde(rename = "autoFix")]
    pub autofix: Option<bool>,
    #[serde(rename = "field")]
    pub field_lint: Option<TextCase>,
    #[serde(rename = "type")]
    pub type_lint: Option<TextCase>,
    #[serde(rename = "enum")]
    pub enum_lint: Option<TextCase>,
    #[serde(rename = "enumValue")]
    pub enum_value_lint: Option<TextCase>,
}

pub fn lint(mut config: Config) -> Result<Config> {
    let lint = config.clone().server.lint;
    if let Some(lint) = lint {
        let autofix = lint.autofix.unwrap_or(false);
        if lint.field_lint.is_some() || lint.default.unwrap_or(false) {
            config = field_lint(config, autofix).to_result()?;
        }
        if lint.type_lint.is_some() || lint.default.unwrap_or(false) {
            config = type_lint(config, autofix).to_result()?;
        }
        if lint.enum_lint.is_some() || lint.default.unwrap_or(false) {
            config = enum_lint(config, autofix).to_result()?;
        }
        if lint.enum_value_lint.is_some() || lint.default.unwrap_or(false) {
            config = enum_value_lint(config, autofix).to_result()?;
        }
    }
    Ok(config)
}

pub fn field_lint(mut config: Config, autofix: bool) -> Valid<Config, String> {
    if let Some(lint) = config.clone().server.lint {
        let config_case = lint.field_lint.unwrap_or(TextCase::CamelCase);
        let target_case = Case::from(config_case);
        let mut errors = Vec::new();
        config.types.iter_mut().for_each(|(_, type_)| {
            for field in type_.fields.clone() {
                let case = field.0.to_case(target_case);
                if case != *field.0 {
                    if autofix {
                        if let Some(field_info) = type_.fields.remove(&field.0) {
                            tracing::warn!(
                                "field {} is renamed to {}",
                                field.0,
                                field.0.to_case(target_case)
                            );
                            type_
                                .fields
                                .insert(field.0.to_case(target_case), field_info);
                        }
                    } else {
                        errors.push(format!(
                            "lint failed for field {}, expected {}",
                            field.0, case
                        ));
                    }
                }
            }
        });
        if !errors.is_empty() {
            return Valid::fail(errors.join("\n"));
        }
    }
    Valid::succeed(config)
}

pub fn type_lint(mut config: Config, autofix: bool) -> Valid<Config, String> {
    if let Some(lint) = config.clone().server.lint {
        let config_case = lint.type_lint.unwrap_or(TextCase::PascalCase);
        let target_case = Case::from(config_case);
        let mut errors = Vec::new();
        config.clone().types.iter_mut().for_each(|(name, _)| {
            if name.to_case(target_case) != *name {
                if autofix {
                    if let Some(type_info) = config.types.remove(name.as_str()) {
                        tracing::warn!("type {} is renamed to {}", name, name.to_case(target_case));
                        config.types.insert(name.to_case(target_case), type_info);
                    }
                } else {
                    errors.push(format!(
                        "lint failed for type {}, expected {}",
                        name,
                        name.to_case(target_case)
                    ));
                }
            }
        });
        if !errors.is_empty() {
            return Valid::fail(errors.join("\n"));
        }
    }
    Valid::succeed(config)
}

pub fn enum_lint(mut config: Config, autofix: bool) -> Valid<Config, String> {
    if let Some(lint) = config.clone().server.lint {
        let config_case = lint.enum_lint.unwrap_or(TextCase::PascalCase);
        let target_case = Case::from(config_case);
        let mut errors = Vec::new();
        config.clone().enums.iter_mut().for_each(|(name, _)| {
            if name.to_case(target_case) != *name {
                if autofix {
                    if let Some(enum_info) = config.enums.remove(name.as_str()) {
                        tracing::warn!("enum {} is renamed to {}", name, name.to_case(target_case));
                        config.enums.insert(name.to_case(target_case), enum_info);
                    }
                } else {
                    errors.push(format!(
                        "lint failed for enum {}, expected {}",
                        name,
                        name.to_case(target_case)
                    ));
                }
            }
        });
    }
    Valid::succeed(config)
}

pub fn enum_value_lint(mut config: Config, autofix: bool) -> Valid<Config, String> {
    if let Some(lint) = config.clone().server.lint {
        let config_case = lint.enum_value_lint.unwrap_or(TextCase::AllCaps);
        let target_case = Case::from(config_case);
        let mut errors = Vec::new();
        config
            .clone()
            .enums
            .iter_mut()
            .for_each(|(enum_name, enum_)| {
                let updated_variants: BTreeSet<_> = enum_
                    .variants
                    .iter()
                    .map(|variant| {
                        let case_name = variant.name.to_case(target_case);
                        if case_name != variant.name {
                            if autofix {
                                tracing::warn!(
                                    "variant {} is renamed to {}",
                                    variant.name,
                                    case_name
                                );
                                // return the variant with the updated name
                                Variant { name: case_name, ..variant.clone() }
                            } else {
                                errors.push(format!(
                                    "lint failed for enum variant {}, expected {}",
                                    variant.name, case_name
                                ));
                                variant.clone()
                            }
                        } else {
                            variant.clone()
                        }
                    })
                    .collect();

                config.enums.remove(enum_name);
                enum_.variants = updated_variants;
                config.enums.insert(enum_name.into(), enum_.clone());
            });
        if !errors.is_empty() {
            return Valid::fail(errors.join("\n"));
        }
    }
    Valid::succeed(config)
}
