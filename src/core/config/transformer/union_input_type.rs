use std::borrow::Cow;

use crate::core::valid::Valid;
use crate::core::{
    config::{Arg, Config, Field, Type},
    transform::Transform,
};

#[derive(Default)]
pub struct UnionInputType;

impl Transform for UnionInputType {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Config) -> Valid<Config, String> {
        // walk over all types and if any field arguments use Union type
        // replace it with set of related fields
        let new_types = config
            .types
            .iter()
            .map(|(name, type_)| {
                let fields = type_
                    .fields
                    .iter()
                    .flat_map(|(name, field)| arguments_to_fields(&config, name, field))
                    .collect();

                let type_ = Type { fields, ..type_.clone() };

                (name.clone(), type_)
            })
            .collect();

        config.types = new_types;

        Valid::succeed(config)
    }
}

/// Converts single field with arguments to possibly multiple fields with
/// arguments based on Union type members.
/// If there is no Union arguments then it will return just the field itself
fn arguments_to_fields(config: &Config, name: &str, field: &Field) -> Vec<(String, Field)> {
    let mut output = Vec::with_capacity(field.args.len());
    let args: Vec<_> = field.args.iter().collect();

    walk_arguments(
        config,
        &args,
        (name.into(), &mut field.clone()),
        &mut output,
    );

    output
}

/// Recursively walks over all arguments for field and setting
/// new fields into the output
fn walk_arguments(
    config: &Config,
    args: &[(&String, &Arg)], // arguments of currently processed field
    (field_name, current_field): (Cow<'_, str>, &mut Field), // new field info
    output: &mut Vec<(String, Field)>, // the result set of fields with their names
) {
    let Some(&(arg_name, arg)) = args.first() else {
        output.push((field_name.into_owned(), current_field.clone()));
        return;
    };

    let args = &args[1..];

    if let Some(union_) = config.find_union(&arg.type_of) {
        // if the type is union walk over all type members and generate new separate
        // field for this variant
        for (i, type_) in union_.types.iter().enumerate() {
            let new_arg = Arg { type_of: type_.clone(), ..arg.clone() };

            current_field.args.insert(arg_name.to_string(), new_arg);
            walk_arguments(
                config,
                args,
                (format!("{field_name}Var{i}").into(), current_field),
                output,
            );
        }
    } else {
        walk_arguments(config, args, (field_name, current_field), output);
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::UnionInputType;
    use crate::core::{config::Config, transform::Transform};
    use crate::core::valid::Validator;
    use crate::core::{config::transformer::AmbiguousType, transform::TransformerOps};

    #[test]
    fn test_output() {
        let config = std::fs::read_to_string(tailcall_fixtures::configs::YAML_UNION).unwrap();
        let config = Config::from_yaml(&config).unwrap();
        let config = UnionInputType
            .pipe(AmbiguousType::default())
            .transform(config)
            .to_result()
            .unwrap();

        assert_snapshot!(config.to_sdl());
    }
}
