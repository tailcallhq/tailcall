use indexmap::IndexMap;

use crate::core::config::Config;
use crate::core::valid::{Valid, Validator};
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
            if !config.types.contains_key(existing_name) {
                Valid::fail(format!(
                    "Type '{}' not found in configuration.",
                    existing_name
                ))
            } else {
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

                Valid::succeed(())
            }
        })
        .map(|_| {
            for type_ in config.types.values_mut() {
                for field_ in type_.fields.values_mut() {
                    // replace type of field.
                    if let Some(suggested_name) = lookup.get(&field_.type_of) {
                        field_.type_of = suggested_name.to_owned();
                    }
                    // replace type of argument.
                    for arg_ in field_.args.values_mut() {
                        if let Some(suggested_name) = lookup.get(&arg_.type_of) {
                            arg_.type_of = suggested_name.clone();
                        }
                    }
                }
            }

            config
        })
    }
}

#[cfg(test)]
mod test {
    use maplit::hashmap;

    use super::RenameTypes;
    use crate::core::config::Config;
    use crate::core::transform::Transform;
    use crate::core::valid::{ValidationError, Validator};

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
            type Post {
                id: ID!
                title: String
                body: String
            }
            type Query {
                posts: [Post] @http(path: "/posts")
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let cfg = RenameTypes::new(
            hashmap! {
                "Query" => "PostQuery",
                "A" => "User",
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
                posts: [Post] @http(path: "/posts")
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
                posts: [Post] @http(path: "/posts")
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let actual = RenameTypes::new(
            hashmap! {
                "Query" => "PostQuery",
                "A" => "User",
                "B" => "User",
                "C" => "User",
            }
            .iter(),
        )
        .transform(config)
        .to_result();

        let b_err = ValidationError::new("Type 'B' not found in configuration.".to_string());
        let c_err = ValidationError::new("Type 'C' not found in configuration.".to_string());
        let expected = Err(b_err.combine(c_err));
        assert_eq!(actual, expected);
    }
}
