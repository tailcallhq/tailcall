use std::collections::HashMap;

use crate::core::config::Config;
use crate::core::valid::Valid;
use crate::core::Transform;

/// Transformer that replaces existing type name
/// with user-suggested names.
pub struct UserSuggestedTypeNames(Vec<SuggestedName>);

pub struct ExistingTypeName(pub String);
pub struct SuggestedTypeName(pub String);

pub struct SuggestedName {
    existing_name: ExistingTypeName,
    suggested_name: SuggestedTypeName,
}

impl SuggestedName {
    pub fn new(existing_name: ExistingTypeName, suggested_name: SuggestedTypeName) -> Self {
        Self { existing_name, suggested_name }
    }
}

impl UserSuggestedTypeNames {
    pub fn new(suggested_names: Vec<SuggestedName>) -> Self {
        Self(suggested_names)
    }

    pub fn replace_type(&self, mut config: Config) -> Valid<Config, String> {
        let mut lookup = HashMap::new();

        for suggest_name in self.0.iter() {
            let suggested_name = suggest_name.suggested_name.0.clone();
            let existing_name = suggest_name.existing_name.0.clone();

            if let Some(type_info) = config.types.remove(&existing_name) {
                config.types.insert(suggested_name.to_string(), type_info);
                lookup.insert(existing_name.clone(), suggested_name.clone());

                // edge case where type is of operation type.
                if config.schema.query == Some(existing_name.clone()) {
                    config.schema.query = Some(suggested_name.clone());
                } else if config.schema.mutation == Some(existing_name.clone()) {
                    config.schema.mutation = Some(suggested_name.clone());
                }
            } else {
                return Valid::fail(format!(
                    "TypeReplacementError: Type '{}' not found in configuration.",
                    existing_name
                ));
            }
        }

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

        Valid::succeed(config)
    }
}

impl Transform for UserSuggestedTypeNames {
    type Value = Config;
    type Error = String;

    fn transform(&self, config: Self::Value) -> Valid<Self::Value, Self::Error> {
        self.replace_type(config)
    }
}

#[cfg(test)]
mod test {
    use super::{ExistingTypeName, SuggestedName, SuggestedTypeName, UserSuggestedTypeNames};
    use crate::core::config::Config;
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    #[test]
    fn test_replace_query_type_with_suggested() {
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
        let cfg = UserSuggestedTypeNames::new(vec![
            SuggestedName::new(
                ExistingTypeName("Query".into()),
                SuggestedTypeName("PostQuery".into()),
            ),
            SuggestedName::new(
                ExistingTypeName("A".into()),
                SuggestedTypeName("User".into()),
            ),
        ])
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
            type PostQuery {
                posts: [Post] @http(path: "/posts")
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();
        let result = UserSuggestedTypeNames::new(vec![SuggestedName::new(
            ExistingTypeName("Query".into()),
            SuggestedTypeName("PostQuery".into()),
        )])
        .transform(config)
        .to_result();
        assert!(result.is_err());
    }
}
