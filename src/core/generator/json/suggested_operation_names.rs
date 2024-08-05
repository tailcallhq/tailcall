use convert_case::{Case, Casing};

use crate::core::config::{Config, GraphQLOperationType};
use crate::core::valid::Valid;
use crate::core::Transform;

/// Transformer that replaces default operation names in the base configuration
/// with user-suggested names.
pub struct UserSuggestedOperationNames<'a> {
    suggest_op_name: &'a str,
    operation_type: &'a GraphQLOperationType,
}

impl<'a> UserSuggestedOperationNames<'a> {
    pub fn new(suggest_op_name: &'a str, operation_type: &'a GraphQLOperationType) -> Self {
        Self { suggest_op_name, operation_type }
    }
}

impl Transform for UserSuggestedOperationNames<'_> {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        let prev_operation_name = self.operation_type.to_string().to_case(Case::Pascal);
        let suggested_name = self.suggest_op_name.to_owned();

        if let Some(type_info) = config.types.remove(&prev_operation_name) {
            config.types.insert(suggested_name.clone(), type_info);
            match self.operation_type {
                GraphQLOperationType::Query => config.schema.query = Some(suggested_name),
                GraphQLOperationType::Mutation => config.schema.mutation = Some(suggested_name),
            }
            Valid::succeed(config)
        } else {
            Valid::fail(format!(
                "Failed to replace type '{}', it was not found in the configuration.",
                prev_operation_name
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use super::UserSuggestedOperationNames;
    use crate::core::config::{Config, GraphQLOperationType};
    use crate::core::valid::Validator;
    use crate::core::Transform;

    #[test]
    fn test_replace_query_type_with_suggested() {
        let sdl = r#"
            schema {
                query: Query
            }
            type Query {
                posts: [Post] @http(path: "/posts")
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();
        let cfg = UserSuggestedOperationNames::new("PostQuery", &GraphQLOperationType::Query)
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
        let result = UserSuggestedOperationNames::new("PostQuery", &GraphQLOperationType::Query)
            .transform(config)
            .to_result();
        assert!(result.is_err());
    }
}
