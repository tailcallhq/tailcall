use indexmap::IndexMap;

use crate::core::config::{Config, Resolver};
use crate::core::valid::{Valid, Validator};
use crate::core::Transform;

#[derive(Clone)]
pub struct FieldName(String);

impl FieldName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

#[derive(Clone)]
pub struct TypeName(String);

impl TypeName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

#[derive(Clone)]
pub struct ArgumentInfo {
    new_argument_name: String,
    field_name: FieldName,
    type_name: TypeName,
}

impl ArgumentInfo {
    pub fn new(
        new_argument_name: String,
        field_name: FieldName,
        type_name: TypeName,
    ) -> ArgumentInfo {
        Self { new_argument_name, field_name, type_name }
    }
}


/// Transformer responsible for renaming the arguments.
/// 
/// old_argument_name => {
///     new_argument_name => new argument name for argument.
///     field_name => the field in which the argument in defined.
///     type_name => the type in which the argument is defined.
/// }
pub struct RenameArgs(IndexMap<String, ArgumentInfo>);

impl RenameArgs {
    pub fn new(arg_rename_map: IndexMap<String, ArgumentInfo>) -> Self {
        Self(arg_rename_map)
    }
}

impl Transform for RenameArgs {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        Valid::from_iter(self.0.iter(), |(existing_arg_name, arg_info)| {
            let type_name = &arg_info.type_name.0;
            let field_name = &arg_info.field_name.0;
            let new_argument_name = &arg_info.new_argument_name;

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

                        let is_rename_op_supported = match &field_.resolver {
                            Some(Resolver::Http(_)) | Some(Resolver::Grpc(_)) | None => true,
                            _ => false,
                        };

                        if !is_rename_op_supported {
                            return Valid::fail(format!(
                                "Cannot rename argument '{}' to '{}' in field '{}' of type '{}'. Renaming is only supported for HTTP and gRPC resolvers.",
                                existing_arg_name, new_argument_name, field_name, type_name
                            ));
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
                user(id: ID!, name: String): JSON
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let arg_info1 = ArgumentInfo::new(
            "userId".to_string(),
            FieldName::new("user"),
            TypeName::new("Query"),
        );
        let arg_info2 = ArgumentInfo::new(
            "userName".to_string(),
            FieldName::new("user"),
            TypeName::new("Query"),
        );

        let rename_args = indexmap::indexmap! {
            "id".to_string() => arg_info1.clone(),
            "name".to_string() => arg_info2.clone(),
        };

        let transformed_config = RenameArgs::new(rename_args)
            .transform(config)
            .to_result()
            .unwrap();

        insta::assert_snapshot!(transformed_config.to_sdl());
    }

    #[test]
    fn test_rename_args_conflict() {
        let sdl = r#"
            type Query {
                user(id: ID!, name: String, userName: String): JSON
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let arg_info = ArgumentInfo::new(
            "userName".to_string(),
            FieldName::new("user"),
            TypeName::new("Query"),
        );

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
