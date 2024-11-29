use tailcall_valid::{Valid, Validator};

use super::BlueprintError;
use crate::core::config::{Config, Field};
use crate::core::mustache::Segment;
use crate::core::scalar::Scalar;
use crate::core::Mustache;

// given a path, it follows path till leaf node and provides callback at leaf
// node.
fn path_validator<'a>(
    config: &Config,
    mut path_iter: impl Iterator<Item = &'a String>,
    type_of: &str,
    leaf_validator: impl Fn(&str) -> bool,
) -> Valid<(), BlueprintError> {
    match config.find_type(type_of) {
        Some(type_def) => match path_iter.next() {
            Some(field) => match type_def.fields.get(field) {
                Some(field_type) => {
                    path_validator(config, path_iter, field_type.type_of.name(), leaf_validator)
                }
                None => Valid::fail(BlueprintError::FieldNotFound(field.to_string())),
            },
            None => Valid::fail(BlueprintError::ValueIsNotOfScalarType(type_of.to_string())),
        },
        None if leaf_validator(type_of) => Valid::succeed(()),
        None => Valid::fail(BlueprintError::TypeNotFoundInConfig(type_of.to_string())),
    }
}

/// Function to validate the arguments in the HTTP resolver.
pub fn validate_argument(
    config: &Config,
    template: Mustache,
    field: &Field,
) -> Valid<(), BlueprintError> {
    let scalar_validator =
        |type_: &str| Scalar::is_predefined(type_) || config.find_enum(type_).is_some();

    Valid::from_iter(template.segments(), |segment| match segment {
        Segment::Expression(expr) if expr.first().map_or(false, |v| v.contains("args")) => {
            match expr.get(1) {
                Some(arg_name) if field.args.get(arg_name).is_some() => {
                    let arg_type_of = field.args.get(arg_name).as_ref().unwrap().type_of.name();
                    path_validator(config, expr.iter().skip(2), arg_type_of, scalar_validator)
                        .trace(arg_name)
                }
                Some(arg_name) => {
                    Valid::fail(BlueprintError::ArgumentNotFound(arg_name.to_string()))
                        .trace(arg_name)
                }
                None => Valid::fail(BlueprintError::TooFewPartsInTemplate),
            }
        }
        _ => Valid::succeed(()),
    })
    .unit()
}

#[cfg(test)]
mod test {
    use tailcall_valid::{Valid, Validator};

    use super::validate_argument;
    use crate::core::blueprint::BlueprintError;
    use crate::core::Mustache;
    use crate::include_config;

    #[test]
    fn test_recursive_case() {
        let config = include_config!("../fixture/recursive-arg.graphql");
        let config = config.unwrap();
        let template = Mustache::parse("{{.args.id.data}}");
        let field = config
            .find_type("Query")
            .and_then(|ty| ty.fields.get("posts"))
            .unwrap();
        let validation_result = validate_argument(&config, template, field);

        assert!(validation_result.is_fail());
        assert_eq!(
            validation_result,
            Valid::fail(BlueprintError::ValueIsNotOfScalarType(
                "PostData".to_string()
            ))
            .trace("id")
        );
    }
}
