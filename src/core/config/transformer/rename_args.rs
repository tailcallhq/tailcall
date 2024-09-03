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
    suggested_names: Vec<String>,
    field_name: FieldName,
    type_name: TypeName,
}

impl ArgumentInfo {
    pub fn new(
        suggested_names: Vec<String>,
        field_name: FieldName,
        type_name: TypeName,
    ) -> ArgumentInfo {
        Self { suggested_names, field_name, type_name }
    }
}

/// arg_name: {
///     suggested_names: Vec<String>, suggested names for argument.
///     field_name: String, name of the field which requires the argument.
///     type_name: String, name of the type where the argument resides.
/// }
pub struct RenameArgs(IndexMap<String, ArgumentInfo>);

impl RenameArgs {
    pub fn new(suggestions: IndexMap<String, ArgumentInfo>) -> Self {
        Self(suggestions)
    }
}

impl Transform for RenameArgs {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        Valid::from_iter(self.0.iter(), |(existing_name, arg_info)| {
            let type_name = &arg_info.type_name.0;
            let field_name = &arg_info.field_name.0;
            config.types.get_mut(type_name)
                .and_then(|type_| type_.fields.get_mut(field_name))
                .and_then(|field_| field_.args.shift_remove(existing_name))
                .map_or_else(
                    || Valid::fail(format!("Argument '{}' not found in type '{}'.", existing_name, type_name)),
                    |arg| {
                        let field_ = config.types.get_mut(type_name)
                            .and_then(|type_| type_.fields.get_mut(field_name))
                            .expect("Field should exist");

                        let new_name = arg_info.suggested_names.iter()
                            .find(|suggested_name| !field_.args.contains_key(*suggested_name))
                            .cloned();

                        match new_name {
                            Some(name) => {
                                field_.args.insert(name.clone(), arg);
                                match field_.resolver.as_mut(){
                                    Some(Resolver::Http(http)) => {
                                        // Note: we shouldn't modify the query params, as modifying them will change the API itself.
                                        http.path = http.path.replace(existing_name, name.as_str());
                                        if let Some(body) = http.body.as_mut() {
                                            *body = body.replace(existing_name, name.as_str());
                                        }
                                    }
                                    Some(Resolver::Grpc(grpc)) => {
                                        if let Some(body) = grpc.body.as_mut() {
                                            if let Some(str_val) = body.as_str() {
                                                *body = serde_json::Value::String(str_val.replace(existing_name, &name));
                                            }
                                        }
                                    }
                                    _ => {
                                        // TODO: handle for other resolvers.
                                    }
                                }

                                Valid::succeed(())
                            },
                            None => {
                                field_.args.insert(existing_name.clone(), arg);
                                Valid::fail(format!(
                                    "Could not rename argument '{}'. All suggested names are already in use.",
                                    existing_name
                                ))
                            }
                        }
                    }
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
            vec!["userId".to_string()],
            FieldName::new("user"),
            TypeName::new("Query"),
        );
        let arg_info2 = ArgumentInfo::new(
            vec!["userName".to_string()],
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
            vec!["userName".to_string()],
            FieldName::new("user"),
            TypeName::new("Query"),
        );

        let rename_args = indexmap::indexmap! {
            "name".to_string() => arg_info,
        };

        let result = RenameArgs::new(rename_args).transform(config).to_result();

        let expected_err = ValidationError::new(
            "Could not rename argument 'name'. All suggested names are already in use.".to_string(),
        );

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), expected_err);
    }
}
