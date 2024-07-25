use super::{to_type, FieldDefinition};
use crate::core::config::{self, Config};
use crate::core::ir::model::{IO, IR};
use crate::core::scalar;
use crate::core::valid::{Valid, Validator};

struct MustachePartsValidator<'a> {
    type_of: &'a config::Type,
    config: &'a Config,
    field: &'a FieldDefinition,
}

impl<'a> MustachePartsValidator<'a> {
    fn new(type_of: &'a config::Type, config: &'a Config, field: &'a FieldDefinition) -> Self {
        Self { type_of, config, field }
    }

    fn validate_type(&self, parts: &[String], is_query: bool) -> Result<(), String> {
        let mut len = parts.len();
        let mut type_of = self.type_of;
        for item in parts {
            let field = type_of.fields.get(item).ok_or_else(|| {
                format!(
                    "no value '{}' found",
                    parts[0..parts.len() - len + 1].join(".").as_str()
                )
            })?;
            let val_type = to_type(field, None);

            if !is_query && val_type.is_nullable() {
                return Err(format!("value '{}' is a nullable type", item.as_str()));
            } else if len == 1 && !scalar::Scalar::is_predefined(val_type.name()) {
                return Err(format!("value '{}' is not of a scalar type", item.as_str()));
            } else if len == 1 {
                break;
            }

            type_of = self
                .config
                .find_type(&field.type_of)
                .ok_or_else(|| format!("no type '{}' found", parts.join(".").as_str()))?;

            len -= 1;
        }

        Ok(())
    }

    fn validate(&self, parts: &[String], is_query: bool) -> Valid<(), String> {
        let config = self.config;
        let args = &self.field.args;

        if parts.len() < 2 {
            return Valid::fail("too few parts in template".to_string());
        }

        let head = parts[0].as_str();
        let tail = parts[1].as_str();

        match head {
            "value" => {
                // all items on parts except the first one
                let tail = &parts[1..];

                if let Err(e) = self.validate_type(tail, is_query) {
                    return Valid::fail(e);
                }
            }
            "args" => {
                // XXX this is a linear search but it's cost is less than that of
                // constructing a HashMap since we'd have 3-4 arguments at max in
                // most cases
                if let Some(arg) = args.iter().find(|arg| arg.name == tail) {
                    if !is_query && arg.of_type.is_list() {
                        return Valid::fail(format!("can't use list type '{tail}' here"));
                    }

                    // we can use non-scalar types in args
                    if !is_query && arg.default_value.is_none() && arg.of_type.is_nullable() {
                        return Valid::fail(format!("argument '{tail}' is a nullable type"));
                    }
                } else {
                    return Valid::fail(format!("no argument '{tail}' found"));
                }
            }
            "vars" => {
                if !config.server.vars.iter().any(|vars| vars.key == tail) {
                    return Valid::fail(format!("var '{tail}' is not set in the server config"));
                }
            }
            "headers" | "env" => {
                // "headers" and "env" refers to values known at runtime, which
                // we can't validate here
            }
            _ => {
                return Valid::fail(format!("unknown template directive '{head}'"));
            }
        }

        Valid::succeed(())
    }
}

impl FieldDefinition {
    pub fn validate_field(&self, type_of: &config::Type, config: &Config) -> Valid<(), String> {
        // XXX we could use `Mustache`'s `render` method with a mock
        // struct implementing the `PathString` trait encapsulating `validation_map`
        // but `render` simply falls back to the default value for a given
        // type if it doesn't exist, so we wouldn't be able to get enough
        // context from that method alone
        // So we must duplicate some of that logic here :(
        let parts_validator = MustachePartsValidator::new(type_of, config, self);

        match &self.resolver {
            Some(IR::IO(IO::Http { req_template, .. })) => {
                Valid::from_iter(req_template.root_url.expression_segments(), |parts| {
                    parts_validator.validate(parts, false).trace("path")
                })
                .and(Valid::from_iter(req_template.query.clone(), |query| {
                    let (_, mustache) = query;

                    Valid::from_iter(mustache.expression_segments(), |parts| {
                        parts_validator.validate(parts, true).trace("query")
                    })
                }))
                .unit()
            }
            Some(IR::IO(IO::GraphQL { req_template, .. })) => {
                Valid::from_iter(req_template.headers.clone(), |(_, mustache)| {
                    Valid::from_iter(mustache.expression_segments(), |parts| {
                        parts_validator.validate(parts, true).trace("headers")
                    })
                })
                .and_then(|_| {
                    if let Some(args) = &req_template.operation_arguments {
                        Valid::from_iter(args, |(_, mustache)| {
                            Valid::from_iter(mustache.expression_segments(), |parts| {
                                parts_validator.validate(parts, true).trace("args")
                            })
                        })
                    } else {
                        Valid::succeed(Default::default())
                    }
                })
                .unit()
            }
            Some(IR::IO(IO::Grpc { req_template, .. })) => {
                Valid::from_iter(req_template.url.expression_segments(), |parts| {
                    parts_validator.validate(parts, false).trace("path")
                })
                .and(
                    Valid::from_iter(req_template.headers.clone(), |(_, mustache)| {
                        Valid::from_iter(mustache.expression_segments(), |parts| {
                            parts_validator.validate(parts, true).trace("headers")
                        })
                    })
                    .unit(),
                )
                .and_then(|_| {
                    if let Some(body) = &req_template.body {
                        if let Some(mustache) = &body.mustache {
                            Valid::from_iter(mustache.expression_segments(), |parts| {
                                parts_validator.validate(parts, true).trace("body")
                            })
                        } else {
                            // TODO: needs review
                            Valid::succeed(Default::default())
                        }
                    } else {
                        Valid::succeed(Default::default())
                    }
                })
                .unit()
            }
            _ => Valid::succeed(()),
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::MustachePartsValidator;
    use crate::core::blueprint::{FieldDefinition, InputFieldDefinition};
    use crate::core::config::{Config, Field, Type};
    use crate::core::valid::Validator;

    fn initialize_test_config_and_field() -> (Config, FieldDefinition) {
        let mut config = Config::default();

        let mut t1_type = Type::default();
        t1_type.fields.insert(
            "numbers".to_owned(),
            Field { type_of: "Int".to_owned(), list: true, ..Default::default() },
        );
        config.types.insert("T1".to_string(), t1_type);

        let type_ = crate::core::blueprint::Type::ListType {
            of_type: Box::new(crate::core::blueprint::Type::NamedType {
                name: "Int".to_string(),
                non_null: false,
            }),
            non_null: false,
        };

        let fld = FieldDefinition {
            name: "f1".to_string(),
            args: vec![InputFieldDefinition {
                name: "q".to_string(),
                of_type: type_,
                default_value: None,
                description: None,
                renames: HashMap::new(),
            }],
            of_type: crate::core::blueprint::Type::NamedType {
                name: "T1".to_string(),
                non_null: false,
            },
            resolver: None,
            directives: vec![],
            description: None,
            default_value: None,
        };

        (config, fld)
    }

    #[test]
    fn test_allow_list_arguments_for_query_type() {
        let (config, field_def) = initialize_test_config_and_field();

        let parts_validator =
            MustachePartsValidator::new(config.types.get("T1").unwrap(), &config, &field_def);
        let validation_result =
            parts_validator.validate(&["args".to_string(), "q".to_string()], true);

        assert!(validation_result.is_succeed())
    }

    #[test]
    fn test_should_not_allow_list_arguments_for_path_variable() {
        let (config, field_def) = initialize_test_config_and_field();

        let parts_validator =
            MustachePartsValidator::new(config.types.get("T1").unwrap(), &config, &field_def);
        let validation_result =
            parts_validator.validate(&["args".to_string(), "q".to_string()], false);

        assert!(validation_result.to_result().is_err())
    }
}
