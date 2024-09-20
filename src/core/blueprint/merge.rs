use std::collections::{BTreeMap, HashSet};

use super::{
    Blueprint, Definition, EnumTypeDefinition, FieldDefinition, InputFieldDefinition,
    InputObjectTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    UnionTypeDefinition,
};
use crate::core::federation::merge::FederatedMerge;
use crate::core::merge_right::MergeRight;
use crate::core::valid::{Valid, Validator};

impl FederatedMerge for InterfaceTypeDefinition {
    fn federated_merge(self, other: Self) -> Valid<Self, String> {
        merge_output_fields(self.fields, other.fields).map(|fields| InterfaceTypeDefinition {
            fields,
            name: other.name,
            description: self.description.merge_right(other.description),
        })
    }
}

impl FederatedMerge for FieldDefinition {
    fn federated_merge(self, other: Self) -> Valid<Self, String> {
        self.of_type
            .wide_merge(other.of_type)
            .fuse(merge_input_fields(self.args, other.args))
            .map(|(of_type, args)| Self {
                name: self.name,
                of_type,
                default_value: self.default_value.or(other.default_value),
                description: self.description.merge_right(other.description),
                resolver: self.resolver.merge_right(other.resolver),
                directives: self.directives.merge_right(other.directives),
                args,
            })
    }
}

impl FederatedMerge for ObjectTypeDefinition {
    fn federated_merge(self, other: Self) -> Valid<Self, String> {
        merge_output_fields(self.fields, other.fields).map(|fields| ObjectTypeDefinition {
            fields,
            name: other.name,
            description: self.description.merge_right(other.description),
            implements: self.implements.merge_right(other.implements),
        })
    }
}

impl FederatedMerge for InputFieldDefinition {
    fn federated_merge(self, other: Self) -> Valid<Self, String> {
        self.of_type
            .narrow_merge(other.of_type)
            .map(|of_type| Self {
                name: self.name,
                of_type,
                default_value: self.default_value.or(other.default_value),
                description: self.description.merge_right(other.description),
            })
    }
}

impl FederatedMerge for InputObjectTypeDefinition {
    // executes intersection merge
    fn federated_merge(self, other: Self) -> Valid<Self, String> {
        merge_input_fields(self.fields, other.fields).map(|fields| InputObjectTypeDefinition {
            fields,
            name: other.name,
            description: self.description.merge_right(other.description),
        })
    }
}

impl FederatedMerge for ScalarTypeDefinition {
    fn federated_merge(self, other: Self) -> Valid<Self, String> {
        // only custom scalar types should appear in the blueprint and they basically
        // equal if they have the same name
        Valid::succeed(self.merge_right(other))
    }
}

impl FederatedMerge for EnumTypeDefinition {
    fn federated_merge(self, other: Self) -> Valid<Self, String> {
        let self_variants: HashSet<_> = self.enum_values.iter().map(|var| &var.name).collect();
        let other_variants: HashSet<_> = other.enum_values.iter().map(|var| &var.name).collect();

        let diff: Vec<_> = self_variants
            .symmetric_difference(&other_variants)
            .collect();

        // By following [spec](https://www.apollographql.com/docs/federation/federated-schemas/composition/#enums)
        // the enum should be merged according to its usage
        // but here we check only if the definitions match exactly
        if diff.is_empty() {
            Valid::fail(format!(
                "Cannot merge Enum definition due to missing variants: {:?} in some definitions",
                diff
            ))
        } else {
            Valid::succeed(EnumTypeDefinition {
                name: self.name,
                directives: self.directives.merge_right(other.directives),
                description: self.description.merge_right(other.description),
                enum_values: self.enum_values,
            })
        }
    }
}

impl FederatedMerge for UnionTypeDefinition {
    fn federated_merge(self, other: Self) -> Valid<Self, String> {
        Valid::succeed(self.merge_right(other))
    }
}

impl FederatedMerge for Definition {
    fn federated_merge(self, other: Self) -> Valid<Self, String> {
        let name = self.name().to_owned();
        if name != other.name() {
            return Valid::fail(format!(
                "Attempt to merge definitions with different names: '{}' <-> '{}'",
                self.name(),
                other.name()
            ));
        }

        match (self, other) {
            (Definition::Interface(left), Definition::Interface(right)) => {
                left.federated_merge(right).map(Definition::Interface)
            }
            (Definition::Object(left), Definition::Object(right)) => {
                left.federated_merge(right).map(Definition::Object)
            }
            (Definition::InputObject(left), Definition::InputObject(right)) => {
                left.federated_merge(right).map(Definition::InputObject)
            }
            (Definition::Scalar(left), Definition::Scalar(right)) => {
                left.federated_merge(right).map(Definition::Scalar)
            }
            (Definition::Enum(left), Definition::Enum(right)) => {
                left.federated_merge(right).map(Definition::Enum)
            }
            (Definition::Union(left), Definition::Union(right)) => {
                left.federated_merge(right).map(Definition::Union)
            }
            _ => Valid::fail("Cannot merge different definitions with the same name".to_string()),
        }
        .trace(&name)
    }
}

impl FederatedMerge for Blueprint {
    fn federated_merge(self, other: Self) -> crate::core::valid::Valid<Self, String> {
        let Blueprint { definitions, schema, server, upstream, telemetry } = self;

        let mut definitions: BTreeMap<_, _> = definitions
            .into_iter()
            .map(|def| (def.name().to_owned(), def))
            .collect();

        Valid::from_iter(other.definitions, |other_definition| {
            match definitions.remove(other_definition.name()) {
                Some(definition) => definition.federated_merge(other_definition),
                None => Valid::succeed(other_definition),
            }
        })
        .map(|mut merged_definitions| {
            merged_definitions.extend(definitions.into_values());

            Self {
                definitions: merged_definitions,
                // all of the other fields are not merged and handled by the router side
                schema,
                server,
                upstream,
                telemetry,
            }
        })
    }
}

fn merge_input_fields(
    left_fields: Vec<InputFieldDefinition>,
    right_fields: Vec<InputFieldDefinition>,
) -> Valid<Vec<InputFieldDefinition>, String> {
    let mut fields: BTreeMap<_, _> = left_fields
        .into_iter()
        .map(|value| (value.name.clone(), value))
        .collect();

    Valid::from_iter(right_fields, |other_field| {
        let name = other_field.name.clone();

        match fields.remove(&other_field.name) {
            Some(field) => field.federated_merge(other_field).map(Some),
            None => {
                if other_field.of_type.is_nullable() {
                    Valid::succeed(None)
                } else {
                    Valid::fail("Input arg is marked as non_null on the right side, but is not present on the left side".to_string())
                }
            },
        }
        .trace(&name)
        })
        .fuse(Valid::from_iter(fields, |(name, field)| {
            if field.of_type.is_nullable() {
                Valid::succeed(())
            } else {
                Valid::fail("Input arg is marked as non_null on the left side, but is not present on the right side".to_string()).trace(&name)
            }
        }))
        .map(|(merged_fields, _)| {
            merged_fields.into_iter().flatten().collect()
        })
}

fn merge_output_fields(
    left_fields: Vec<FieldDefinition>,
    right_fields: Vec<FieldDefinition>,
) -> Valid<Vec<FieldDefinition>, String> {
    let mut fields: BTreeMap<_, _> = left_fields
        .into_iter()
        .map(|value| (value.name.clone(), value))
        .collect();

    Valid::from_iter(right_fields, |other_field| {
        let name = other_field.name.clone();

        match fields.remove(&name) {
            Some(field) => field.federated_merge(other_field).trace(&name),
            None => Valid::succeed(other_field),
        }
    })
    .map(|mut merged_fields| {
        merged_fields.extend(fields.into_values());

        merged_fields
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use insta::assert_snapshot;
    use tailcall_fixtures::configs::federation;

    use crate::core::blueprint::Blueprint;
    use crate::core::config::Config;
    use crate::core::federation::merge::FederatedMerge;
    use crate::core::valid::Validator;

    fn load_blueprint(path: &str) -> Blueprint {
        let sdl = fs::read_to_string(path).unwrap();
        let config = Config::from_sdl(&sdl).to_result().unwrap();

        Blueprint::try_from(&config.into()).unwrap()
    }

    #[test]
    fn merge_router_and_subgraphs() {
        let subgraph_users = load_blueprint(federation::SUBGRAPH_USERS);
        let subgraph_posts = load_blueprint(federation::SUBGRAPH_POSTS);
        let router = load_blueprint(federation::ROUTER);

        let result = router
            .federated_merge(subgraph_users)
            .to_result()
            .unwrap()
            .federated_merge(subgraph_posts)
            .to_result()
            .unwrap();

        let sdl = result.to_schema().sdl();

        assert_snapshot!(sdl);
    }
}
