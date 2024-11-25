use std::collections::HashSet;

use indexmap::IndexMap;
use tailcall_valid::{Valid, Validator};

use crate::core::config::Config;
use crate::core::Transform;

/// A transformer that renames existing types by replacing them with suggested
/// names.
pub struct RenameTypes(IndexMap<String, String>);

impl RenameTypes {
    pub fn new<I: Iterator<Item = (S, S)>, S: ToString>(suggested_names: I) -> Self {
        Self(
            suggested_names
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
        )
    }
}

impl Transform for RenameTypes {
    type Value = Config;
    type Error = String;

    fn transform(&self, config: Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut config = config;
        let mut lookup = IndexMap::new();

        // Ensure all types exist in the configuration
        Valid::from_iter(self.0.iter(), |(existing_name, suggested_name)| {
            if config.types.contains_key(existing_name)
                || config.enums.contains_key(existing_name)
                || config.unions.contains_key(existing_name)
            {
                // handle for the types.
                if let Some(type_info) = config.types.remove(existing_name) {
                    config.types.insert(suggested_name.to_string(), type_info);
                    lookup.insert(existing_name.clone(), suggested_name.clone());

                    // edge case where type is of operation type.
                    if config.schema.query == Some(existing_name.clone()) {
                        config.schema.query = Some(suggested_name.clone());
                    } else if config.schema.mutation == Some(existing_name.clone()) {
                        config.schema.mutation = Some(suggested_name.clone());
                    }
                }

                // handle for the enums.
                if let Some(type_info) = config.enums.remove(existing_name) {
                    config.enums.insert(suggested_name.to_string(), type_info);
                    lookup.insert(existing_name.clone(), suggested_name.clone());
                }

                // handle for the union.
                if let Some(type_info) = config.unions.remove(existing_name) {
                    config.unions.insert(suggested_name.to_string(), type_info);
                    lookup.insert(existing_name.clone(), suggested_name.clone());
                }

                Valid::succeed(())
            } else {
                Valid::fail(format!(
                    "Type '{}' not found in configuration.",
                    existing_name
                ))
            }
        })
        .map(|_| {
            for type_ in config.types.values_mut() {
                for field_ in type_.fields.values_mut() {
                    // replace type of field.
                    if let Some(suggested_name) = lookup.get(field_.type_of.name()) {
                        field_.type_of =
                            field_.type_of.clone().with_name(suggested_name.to_owned());
                    }
                    // replace type of argument.
                    for arg_ in field_.args.values_mut() {
                        if let Some(suggested_name) = lookup.get(arg_.type_of.name()) {
                            arg_.type_of =
                                arg_.type_of.clone().with_name(suggested_name.to_owned());
                        }
                    }
                }

                // replace in interface.
                type_.implements = type_
                    .implements
                    .iter()
                    .map(|interface_type_name| {
                        lookup
                            .get(interface_type_name)
                            .cloned()
                            .unwrap_or_else(|| interface_type_name.to_owned())
                    })
                    .collect();
            }

            // replace in the union as well.
            for union_type_ in config.unions.values_mut() {
                // Collect changes to be made
                let mut types_to_remove = HashSet::new();
                let mut types_to_add = HashSet::new();

                for type_name in union_type_.types.iter() {
                    if let Some(new_type_name) = lookup.get(type_name) {
                        types_to_remove.insert(type_name.clone());
                        types_to_add.insert(new_type_name.clone());
                    }
                }
                // Apply changes
                for type_name in types_to_remove {
                    union_type_.types.remove(&type_name);
                }

                for type_name in types_to_add {
                    union_type_.types.insert(type_name);
                }
            }

            // replace in union as well.
            for union_type_ in config.unions.values_mut() {
                union_type_.types = union_type_
                    .types
                    .iter()
                    .map(|type_name| {
                        lookup
                            .get(type_name)
                            .cloned()
                            .unwrap_or_else(|| type_name.to_owned())
                    })
                    .collect();
            }

            config
        })
    }
}

#[cfg(test)]
mod test {
    use indexmap::IndexMap;
    use maplit::hashmap;
    use tailcall_valid::{ValidationError, Validator};

    use super::RenameTypes;
    use crate::core::config::Config;
    use crate::core::transform::Transform;

    #[test]
    fn test_rename_type() {
        let sdl = r#"
            schema {
                query: Query
            }
            type A {
                id: ID!
                name: String
            }
            type B {
                name: String
                username: String
            }
            union FooBar = A | B
            type Post {
                id: ID!
                title: String
                body: String
            }
            enum Status {
                PENDING
                STARTED,
                COMPLETED
            }
            type Query {
                posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
            }
            type Mutation {
              createUser(user: B!): A @http(method: POST, url: "http://jsonplaceholder.typicode.com/users", body: "{{args.user}}")
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let cfg = RenameTypes::new(
            hashmap! {
                "Query" => "PostQuery",
                "A" => "User",
                "B" => "InputUser",
                "Mutation" => "UserMutation",
                "Status" => "TaskStatus"
            }
            .iter(),
        )
        .transform(config)
        .to_result()
        .unwrap();

        insta::assert_snapshot!(cfg.to_sdl())
    }

    #[test]
    fn test_should_raise_error_when_operation_type_name_is_different() {
        let sdl = r#"
            schema {
                query: PostQuery
            }
            type Post {
                id: ID
                title: String
            }
            type PostQuery {
                posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let result = RenameTypes::new(hashmap! {"Query" =>  "PostQuery"}.iter())
            .transform(config)
            .to_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_should_raise_error_when_type_not_found() {
        let sdl = r#"
            schema {
                query: Query
            }
            type A {
                id: ID!
                name: String
            }
            type Post {
                id: ID!
                title: String
                body: String
            }
            type Query {
                posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let mut suggested_names = IndexMap::new();
        suggested_names.insert("Query", "PostQuery");
        suggested_names.insert("A", "User");
        suggested_names.insert("B", "User");
        suggested_names.insert("C", "User");

        let actual = RenameTypes::new(suggested_names.iter())
            .transform(config)
            .to_result();

        let b_err = ValidationError::new("Type 'B' not found in configuration.".to_string());
        let c_err = ValidationError::new("Type 'C' not found in configuration.".to_string());
        let expected = Err(b_err.combine(c_err));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_inferface_rename() {
        let sdl = r#"
            schema {
                query: Query
            }
            interface Node {
                id: ID
            }
            type Post implements Node {
                id: ID
                title: String
            }
            type Query {
                posts: [Post] @http(url: "/posts")
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let result = RenameTypes::new(hashmap! {"Node" =>  "NodeTest"}.iter())
            .transform(config)
            .to_result()
            .unwrap();
        insta::assert_snapshot!(result.to_sdl())
    }
}
