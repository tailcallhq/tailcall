use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::core::config::{Arg, Config, Field, Type};
use crate::core::transform::Transform;
use crate::core::valid::Valid;

/// Transforms unions inside the input types by replacing actual unions
/// with multiple variants of the parent type, with each field resolved
/// as one of the union's type members.
///
/// Algorithm explanation:
///
/// For every type, check its field's arguments with the following steps:
///     1. Recursively check if the argument's type itself or its fields use
///        union types in their definitions.
///     2. For every such type in the recursive tree, replace the type with
///        multiple new types where the union field is replaced by concrete
///        union type variants.
///     3. Bubble up such replacements up to the initial argument type.
///     4. Replace the type (the one with the union argument) with the new type
///        where additional fields were added instead of the original field,
///        with the union type replaced by concrete type variants.
#[derive(Default)]
pub struct UnionInputType;

impl Transform for UnionInputType {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Config) -> Valid<Config, String> {
        let visitor = Visitor::new(&config);

        let new_types = visitor.visit();

        config.types = new_types;

        Valid::succeed(config)
    }
}

/// Specifies if the type should be represented as a union of types
/// if it is a union itself or if any nested field is a union.
#[derive(Debug)]
enum UnionPresence {
    NoUnion,
    Union(Vec<String>),
}

struct Visitor<'cfg> {
    /// Original config
    config: &'cfg Config,
    /// Result types that will replace original config.types
    new_types: BTreeMap<String, Type>,
    // maps type name to UnionPresence
    union_presence: HashMap<&'cfg String, UnionPresence>,
    // track visited types
    visited_types: Vec<&'cfg String>,
}

impl<'cfg> Visitor<'cfg> {
    fn new(config: &'cfg Config) -> Self {
        Self {
            config,
            union_presence: HashMap::new(),
            new_types: BTreeMap::new(),
            visited_types: Vec::new(),
        }
    }

    /// Walks over all the types to see if we need to add new types
    /// or replace the field set for the type if any of the arguments use
    /// a union somewhere down the fields tree.
    fn visit(mut self) -> BTreeMap<String, Type> {
        for (type_name, type_) in &self.config.types {
            let fields = type_
                .fields
                .iter()
                .flat_map(|(field_name, field)| {
                    // first need to collect info about possible
                    // unions presence starting from this field
                    self.collect_nested_unions_for_args(field);

                    // then convert if needed field's arguments with unions into
                    // multiple fields with specific variant
                    self.map_args_to_fields(field_name, field)
                })
                .collect();

            // new type will replace existing type in the config
            self.new_types
                .insert(type_name.clone(), Type { fields, ..type_.clone() });
        }

        self.new_types
    }

    /// Walks over the field's arguments and fills union_presence info
    fn collect_nested_unions_for_args(&mut self, field: &'cfg Field) {
        field
            .args
            .values()
            .for_each(|arg| self.collect_nested_unions_for_type(&arg.type_of))
    }

    /// Recursively walks over nested types and fills union_presence info
    fn collect_nested_unions_for_type(&mut self, type_name: &'cfg String) {
        if self.union_presence.contains_key(type_name) || self.visited_types.contains(&type_name) {
            return;
        }
        // avoid endless recurssion
        self.visited_types.push(type_name);

        if let Some(union_) = self.config.unions.get(type_name) {
            // if the type is union process the nested types recursively
            for type_name in &union_.types {
                self.collect_nested_unions_for_type(type_name);
            }

            let mut types = BTreeSet::new();

            for type_name in &union_.types {
                if let Some(UnionPresence::Union(union_types)) = self.union_presence.get(type_name)
                {
                    // union type members could be the union itself or could be the type
                    // that has nested unions
                    types.extend(union_types.clone());
                } else {
                    types.insert(type_name.clone());
                }
            }

            self.union_presence
                .insert(type_name, UnionPresence::Union(types.into_iter().collect()));
        } else if let Some(type_) = self.config.types.get(type_name) {
            // first, recursively walk over nested fields to see if there any nested unions
            for field in type_.fields.values() {
                self.collect_nested_unions_for_type(&field.type_of);
            }

            // store any fields that contain union
            let mut union_fields = Vec::new();

            // then again loop over fields and check if there any fields that are resolved
            // to multiple types. As separate loop to bypass borrow checker
            for (field_name, field) in &type_.fields {
                if let Some(UnionPresence::Union(union_types)) =
                    self.union_presence.get(&field.type_of)
                {
                    union_fields.push((field_name, union_types));
                }
            }

            if union_fields.is_empty() {
                // if there are no union nested types just mark it as NoUnion
                self.union_presence
                    .insert(type_name, UnionPresence::NoUnion);
            } else {
                // if there are union types we need to create new types
                // without unions and add to list of result types
                // and union_presence info
                let union_types = self.create_types_from_union(type_name, type_, union_fields);

                self.union_presence.insert(
                    type_name,
                    UnionPresence::Union(
                        union_types
                            .iter()
                            .map(|(type_name, _)| type_name.clone())
                            .collect(),
                    ),
                );

                self.new_types.extend(union_types);
            }
        }
    }

    /// Converts single field with arguments to possibly multiple fields with
    /// arguments based on Union type members.
    /// If there is no Union arguments then it will return just the field itself
    fn map_args_to_fields(&self, name: &str, field: &Field) -> Vec<(String, Field)> {
        let mut output = Vec::with_capacity(field.args.len());
        let args: Vec<_> = field.args.iter().collect();

        self.walk_arguments(&args, (name.into(), &mut field.clone()), &mut output);

        output
    }

    /// Recursively walks over all arguments and creates
    /// new fields with the single argument replaced with
    /// one of the union type member
    fn walk_arguments(
        &self,
        args: &[(&String, &Arg)], // arguments of currently processed field
        (field_name, current_field): (Cow<'_, str>, &mut Field), // new field info
        output: &mut Vec<(String, Field)>, // the result set of fields with their names
    ) {
        let Some(&(arg_name, arg)) = args.first() else {
            output.push((field_name.into_owned(), current_field.clone()));
            return;
        };

        let args = &args[1..];

        if let Some(UnionPresence::Union(union_types)) = self.union_presence.get(&arg.type_of) {
            // if the type is union walk over all type members and generate new separate
            // field for this variant
            for (i, type_) in union_types.iter().enumerate() {
                let new_arg = Arg { type_of: type_.clone(), ..arg.clone() };

                current_field.args.insert(arg_name.to_string(), new_arg);
                self.walk_arguments(
                    args,
                    (format!("{field_name}Var{i}").into(), current_field),
                    output,
                );
            }
        } else {
            self.walk_arguments(args, (field_name, current_field), output);
        }
    }

    /// Creates new mirror types for the original type that contained unions.
    /// All the fields that resolved to union type are replaced with specific
    /// union type member one at a time.
    fn create_types_from_union(
        &self,
        type_name: &str,
        type_: &Type,
        union_fields: Vec<(&String, &Vec<String>)>,
    ) -> Vec<(String, Type)> {
        fn inner_create(
            type_name: String,                        // name of the new type to set
            base_type: Type,                          // current representation of the type
            union_fields: &[(&String, &Vec<String>)], // list of fields that are union
            result: &mut Vec<(String, Type)>,         /* the result list of new types with their
                                                       * names */
        ) {
            let Some((field_name, union_types)) = union_fields.first().as_ref() else {
                result.push((type_name, base_type));

                return;
            };

            let union_fields = &union_fields[1..];

            for (i, union_type) in union_types.iter().enumerate() {
                let type_name = format!("{type_name}__{field_name}{i}");
                let mut new_type = base_type.clone();

                let field = new_type
                    .fields
                    .get_mut(*field_name)
                    .expect("Only available fields could be in list of union_fields");

                field.type_of.clone_from(union_type);

                inner_create(type_name, new_type, union_fields, result);
            }
        }

        let mut new_types = Vec::new();

        inner_create(
            type_name.to_owned(),
            type_.clone(),
            &union_fields,
            &mut new_types,
        );

        new_types
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::UnionInputType;
    use crate::core::config::Config;
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    #[test]
    fn test_union() {
        let config = std::fs::read_to_string(tailcall_fixtures::configs::YAML_UNION).unwrap();
        let config = Config::from_yaml(&config).unwrap();
        let config = UnionInputType.transform(config).to_result().unwrap();

        assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_union_in_type() {
        let config =
            std::fs::read_to_string(tailcall_fixtures::configs::YAML_UNION_IN_TYPE).unwrap();
        let config = Config::from_yaml(&config).unwrap();
        let config = UnionInputType.transform(config).to_result().unwrap();

        assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_nested_unions() {
        let config =
            std::fs::read_to_string(tailcall_fixtures::configs::YAML_NESTED_UNIONS).unwrap();
        let config = Config::from_yaml(&config).unwrap();
        let config = UnionInputType.transform(config).to_result().unwrap();

        assert_snapshot!(config.to_sdl());
    }
    #[test]
    fn test_recurssive_input() {
        let config =
            std::fs::read_to_string(tailcall_fixtures::configs::YAML_RECURSSIVE_INPUT).unwrap();
        let config = Config::from_yaml(&config).unwrap();
        let config = UnionInputType.transform(config).to_result().unwrap();

        assert_snapshot!(config.to_sdl());
    }
}
