use indexmap::IndexMap;

use crate::core::config::{Config, Resolver};
use crate::core::valid::{Valid, Validator};
use crate::core::Transform;

#[derive(Clone)]
pub struct Location {
    pub new_argument_name: String,
    pub field_name: String,
    pub type_name: String,
}

/// Transformer responsible for renaming the arguments.
///
/// old_argument_name => {
///     new_argument_name => new argument name for argument.
///     field_name => the field in which the argument in defined.
///     type_name => the type in which the argument is defined.
/// }
pub struct RenameArgs(IndexMap<String, Location>);

impl RenameArgs {
    pub fn new(arg_rename_map: IndexMap<String, Location>) -> Self {
        Self(arg_rename_map)
    }
}

impl Transform for RenameArgs {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        Valid::from_iter(self.0.iter(), |(existing_arg_name, location)| {
            // note: we can use expect on Location type as this type it's impossible to call this function without location being not set.
            let type_name = location.type_name.as_str();
            let field_name = location.field_name.as_str();
            let new_argument_name = location.new_argument_name.as_str();

            config
                .types
                .get_mut(type_name)
                .and_then(|type_| type_.fields.get_mut(field_name))
                .map_or_else(
                    || Valid::fail(format!(
                        "Cannot rename argument as Field '{}' not found in type '{}'.",
                        existing_arg_name, type_name
                    )),
                    |field_| {
                        if field_.args.contains_key(new_argument_name) {
                            return Valid::fail(format!(
                                "Cannot rename argument from '{}' to '{}' as it already exists in field '{}' of type '{}'.",
                                existing_arg_name, new_argument_name, field_name, type_name
                            ));
                        }

                        if !matches!(&field_.resolver, Some(Resolver::Http(_)) | Some(Resolver::Grpc(_)) | None) {
                            return Valid::fail(format!(
                                "Cannot rename argument '{}' to '{}' in field '{}' of type '{}'. Renaming is only supported for HTTP and gRPC resolvers.",
                                existing_arg_name, new_argument_name, field_name, type_name
                            ));
                        }

                        if let Some(Resolver::Http(http)) = &field_.resolver {
                            if http.query.iter().any(|q| &q.key == existing_arg_name) {
                                return Valid::fail(format!(
                                    "Cannot rename argument '{}' to '{}' in field '{}' of type '{}'. Renaming of query parameters is not allowed.",
                                    existing_arg_name, new_argument_name, field_name, type_name
                                ));
                            }
                        }

                        if let Some(arg) = field_.args.shift_remove(existing_arg_name) {
                            field_.args.insert(new_argument_name.to_owned(), arg);
                            if let Some(resolver) = &mut field_.resolver {
                                match resolver {
                                    Resolver::Http(http) => {
                                        http.path = http.path.replace(existing_arg_name, new_argument_name);
                                        if let Some(body) = &mut http.body {
                                            *body = body.replace(existing_arg_name, new_argument_name);
                                        }
                                    }
                                    Resolver::Grpc(grpc) => {
                                        if let Some(body) = &mut grpc.body {
                                            if let Some(str_val) = body.as_str() {
                                                *body = serde_json::Value::String(str_val.replace(existing_arg_name, new_argument_name));
                                            }
                                        }
                                    }
                                    _ => {} // TODO: presently only HTTP & gRPC resolvers are supported, later on add support for rest of resolvers.
                                }
                            }
                            Valid::succeed(())

                        }else{
                            Valid::fail(format!(
                                "Cannot rename argument '{}' as it does not exist in field '{}' of type '{}'.",
                                existing_arg_name, field_name, type_name
                            ))
                        }
                    },
                )
        })
        .map(|_| config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::valid::ValidationError;

    #[test]
    fn test_rename_args() {
        let sdl = r#"
            type Query {
                user(id: ID!): JSON @http(path: "https://jsonplaceholder.typicode.com/users/{{.args.id}}")
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let arg_info1 = Location {
            field_name: "user".to_string(),
            new_argument_name: "userId".to_string(),
            type_name: "Query".to_string(),
        };

        let rename_args = indexmap::indexmap! {
            "id".to_string() => arg_info1.clone(),
        };

        let transformed_config = RenameArgs::new(rename_args)
            .transform(config)
            .to_result()
            .unwrap();

        insta::assert_snapshot!(transformed_config.to_sdl());
    }

    #[test]
    fn test_fail_query_parameter_rename() {
        let sdl = r#"
            type Query {
                user(id: ID!, name: String): JSON @http(path: "https://jsonplaceholder.typicode.com/users", query: [{key: "id", value: "{{.args.id}}"}])
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let arg_info1 = Location {
            field_name: "user".to_string(),
            new_argument_name: "userId".to_string(),
            type_name: "Query".to_string(),
        };

        let rename_args = indexmap::indexmap! {
            "id".to_string() => arg_info1,
        };

        let result = RenameArgs::new(rename_args).transform(config).to_result();
        let expected_err = ValidationError::new(
            "Cannot rename argument 'id' to 'userId' in field 'user' of type 'Query'. Renaming of query parameters is not allowed.".into()
        );

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), expected_err);
    }

    #[test]
    fn test_body_param() {
        let sdl = r#"
            schema {
                query: Query
                mutation: Mutation
            }
            type Query {
                hello: String! @expr(body: "test")
            }
            type Mutation {
              createPost(input: JSON): JSON! @http(path: "/posts", body: "{{.args.input}}", method: "POST")
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let arg_info1 = Location {
            field_name: "createPost".to_string(),
            new_argument_name: "createPostInput".to_string(),
            type_name: "Mutation".to_string(),
        };

        let rename_args = indexmap::indexmap! {
            "input".to_string() => arg_info1,
        };

        let result = RenameArgs::new(rename_args)
            .transform(config)
            .to_result()
            .unwrap();
        insta::assert_snapshot!(result.to_sdl());
    }

    #[test]
    fn test_rename_args_conflict() {
        let sdl = r#"
            type Query {
                user(id: ID!, name: String, userName: String): JSON
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let arg_info = Location {
            field_name: "user".to_string(),
            new_argument_name: "userName".to_string(),
            type_name: "Query".to_string(),
        };

        let rename_args = indexmap::indexmap! {
            "name".to_string() => arg_info,
        };

        let result = RenameArgs::new(rename_args).transform(config).to_result();

        let expected_err = ValidationError::new(
            "Cannot rename argument from 'name' to 'userName' as it already exists in field 'user' of type 'Query'.".into(),
        );

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), expected_err);
    }
}
