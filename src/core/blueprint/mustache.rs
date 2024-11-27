use tailcall_valid::{Valid, Validator};

use super::{BlueprintError, FieldDefinition};
use crate::core::config::{self, Config};
use crate::core::directive::DirectiveCodec;
use crate::core::ir::model::{IO, IR};
use crate::core::scalar;

struct MustachePartsValidator<'a> {
    type_of: &'a config::Type,
    config: &'a Config,
    field: &'a FieldDefinition,
}

impl<'a> MustachePartsValidator<'a> {
    fn new(type_of: &'a config::Type, config: &'a Config, field: &'a FieldDefinition) -> Self {
        Self { type_of, config, field }
    }

    fn validate_type(&self, parts: &[String], is_query: bool) -> Result<(), BlueprintError> {
        let mut len = parts.len();
        let mut type_of = self.type_of;
        for item in parts {
            let field = type_of.fields.get(item).ok_or_else(|| {
                BlueprintError::NoValueFound(parts[0..parts.len() - len + 1].join("."))
            })?;
            let val_type = &field.type_of;

            if !is_query && val_type.is_nullable() {
                return Err(BlueprintError::ValueIsNullableType(item.clone()));
            } else if len == 1 && !scalar::Scalar::is_predefined(val_type.name()) {
                return Err(BlueprintError::ValueIsNotOfScalarType(item.clone()));
            } else if len == 1 {
                break;
            }

            type_of = self
                .config
                .find_type(val_type.name())
                .ok_or_else(|| BlueprintError::NoTypeFound(parts.join(".")))?;

            len -= 1;
        }

        Ok(())
    }

    fn validate(&self, parts: &[String], is_query: bool) -> Valid<(), BlueprintError> {
        let config = self.config;
        let args = &self.field.args;

        if parts.len() < 2 {
            return Valid::fail(BlueprintError::TooFewPartsInTemplate);
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
                        return Valid::fail(BlueprintError::CantUseListTypeHere(tail.to_string()));
                    }

                    // we can use non-scalar types in args
                    if !is_query && arg.default_value.is_none() && arg.of_type.is_nullable() {
                        return Valid::fail(BlueprintError::ArgumentIsNullableType(
                            tail.to_string(),
                        ));
                    }
                } else {
                    return Valid::fail(BlueprintError::ArgumentNotFound(tail.to_string()));
                }
            }
            "vars" => {
                if !config.server.vars.iter().any(|vars| vars.key == tail) {
                    return Valid::fail(BlueprintError::VarNotSetInServerConfig(tail.to_string()));
                }
            }
            "headers" | "env" => {
                // "headers" and "env" refers to values known at runtime, which
                // we can't validate here
            }
            _ => {
                return Valid::fail(BlueprintError::UnknownTemplateDirective(head.to_string()));
            }
        }

        Valid::succeed(())
    }

    fn validate_resolver(&self, resolver: &IR) -> Valid<(), BlueprintError> {
        match resolver {
            IR::Merge(resolvers) => {
                Valid::from_iter(resolvers, |resolver| self.validate_resolver(resolver)).unit()
            }
            IR::IO(IO::Http { req_template, .. }) => {
                Valid::from_iter(req_template.root_url.expression_segments(), |parts| {
                    self.validate(parts, false).trace("path")
                })
                .and(Valid::from_iter(req_template.query.clone(), |query| {
                    let mustache = &query.value;

                    Valid::from_iter(mustache.expression_segments(), |parts| {
                        self.validate(parts, true).trace("query")
                    })
                }))
                .unit()
                .trace(config::Http::trace_name().as_str())
            }
            IR::IO(IO::GraphQL { req_template, .. }) => {
                Valid::from_iter(req_template.headers.clone(), |(_, mustache)| {
                    Valid::from_iter(mustache.expression_segments(), |parts| {
                        self.validate(parts, true).trace("headers")
                    })
                })
                .and_then(|_| {
                    if let Some(args) = &req_template.operation_arguments {
                        Valid::from_iter(args, |(_, mustache)| {
                            Valid::from_iter(mustache.expression_segments(), |parts| {
                                self.validate(parts, true).trace("args")
                            })
                        })
                    } else {
                        Valid::succeed(Default::default())
                    }
                })
                .unit()
                .trace(config::GraphQL::trace_name().as_str())
            }
            IR::IO(IO::Grpc { req_template, .. }) => {
                Valid::from_iter(req_template.url.expression_segments(), |parts| {
                    self.validate(parts, false).trace("path")
                })
                .and(
                    Valid::from_iter(req_template.headers.clone(), |(_, mustache)| {
                        Valid::from_iter(mustache.expression_segments(), |parts| {
                            self.validate(parts, true).trace("headers")
                        })
                    })
                    .unit(),
                )
                .and_then(|_| {
                    if let Some(body) = &req_template.body {
                        if let Some(mustache) = &body.mustache {
                            Valid::from_iter(mustache.expression_segments(), |parts| {
                                self.validate(parts, true).trace("body")
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
                .trace(config::Grpc::trace_name().as_str())
            }
            // TODO: add validation for @expr
            _ => Valid::succeed(()),
        }
    }
}

impl FieldDefinition {
    pub fn validate_field(
        &self,
        type_of: &config::Type,
        config: &Config,
    ) -> Valid<(), BlueprintError> {
        // XXX we could use `Mustache`'s `render` method with a mock
        // struct implementing the `PathString` trait encapsulating `validation_map`
        // but `render` simply falls back to the default value for a given
        // type if it doesn't exist, so we wouldn't be able to get enough
        // context from that method alone
        // So we must duplicate some of that logic here :(
        let parts_validator = MustachePartsValidator::new(type_of, config, self);

        match &self.resolver {
            Some(resolver) => parts_validator.validate_resolver(resolver),
            None => Valid::succeed(()),
        }
    }
}

#[cfg(test)]
mod test {
    use tailcall_valid::Validator;

    use super::MustachePartsValidator;
    use crate::core::blueprint::{FieldDefinition, InputFieldDefinition};
    use crate::core::config::{self, Config, Field};
    use crate::core::Type;

    fn initialize_test_config_and_field() -> (Config, FieldDefinition) {
        let mut config = Config::default();

        let mut t1_type = config::Type::default();
        t1_type.fields.insert(
            "numbers".to_owned(),
            Field {
                type_of: Type::from("Int".to_owned()).into_list(),
                ..Default::default()
            },
        );
        config.types.insert("T1".to_string(), t1_type);

        let type_ = Type::List {
            of_type: Box::new(Type::Named { name: "Int".to_string(), non_null: false }),
            non_null: false,
        };

        let fld = FieldDefinition {
            name: "f1".to_string(),
            args: vec![InputFieldDefinition {
                name: "q".to_string(),
                of_type: type_,
                default_value: None,
                description: None,
            }],
            of_type: Type::Named { name: "T1".to_string(), non_null: false },
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
