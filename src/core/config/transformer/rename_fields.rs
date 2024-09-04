use indexmap::IndexMap;

use super::TypeName;
use crate::core::config::Config;
use crate::core::valid::{Valid, Validator};
use crate::core::Transform;

#[derive(Clone)]
pub struct FieldInfo {
    suggestions: Vec<String>,
    type_name: TypeName,
}

impl FieldInfo {
    pub fn new(suggestions: Vec<String>, type_name: TypeName) -> Self {
        Self { suggestions, type_name }
    }
}

#[derive(Clone)]
pub struct RenameFields(IndexMap<String, FieldInfo>);

impl RenameFields {
    pub fn new(mappings: IndexMap<String, FieldInfo>) -> Self {
        Self(mappings)
    }
}

impl Transform for RenameFields {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut value: Self::Value) -> Valid<Self::Value, Self::Error> {
        Valid::from_iter(self.0.iter(), |(field_name, field_info)| {
            let type_name = field_info.type_name.as_str();

            if let Some(type_def) = value.types.get_mut(type_name) {
                let suggested_field_name = field_info.suggestions.iter().find(|name| !type_def.fields.contains_key(*name));

                match suggested_field_name {
                    Some(suggested_name) => {
                        if let Some(field_def) = type_def.fields.remove(field_name) {
                            type_def.fields.insert(suggested_name.to_owned(), field_def);
                            Valid::succeed(())
                        } else {
                            Valid::fail(format!("Field '{}' not found in type '{}'.", field_name, type_name))
                        }
                    }
                    None => Valid::fail(format!(
                        "Could not rename field '{}' in type '{}'. All suggested names are already in use.",
                        field_name, type_name
                    )),
                }
            } else {
                Valid::fail(format!("Type '{}' not found.", type_name))
            }
        })
        .map(|_| value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::valid::ValidationError;

    #[test]
    fn test_successful_rename() {
        let sdl = r#"
            type User {
                old_name: String
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let mut mappings = IndexMap::new();
        mappings.insert(
            "old_name".to_string(),
            FieldInfo::new(vec!["new_name".to_string()], TypeName::new("User")),
        );
        let transformed_config = RenameFields::new(mappings)
            .transform(config)
            .to_result()
            .unwrap();

        let user_type = transformed_config.find_type("User").unwrap();

        assert!(user_type.fields.contains_key("new_name"));
        assert!(!user_type.fields.contains_key("old_name"));
    }

    #[test]
    fn test_rename_with_multiple_suggestions() {
        let sdl = r#"
            type User {
                old_name: String
                new_name: String
            }
        "#;

        let config = Config::from_sdl(sdl).to_result().unwrap();

        let mut mappings = IndexMap::new();
        mappings.insert(
            "old_name".to_string(),
            FieldInfo::new(
                vec!["new_name".to_string(), "updated_name".to_string()],
                TypeName::new("User"),
            ),
        );
        let transformed_config = RenameFields::new(mappings)
            .transform(config)
            .to_result()
            .unwrap();

        let user_type = transformed_config.find_type("User").unwrap();

        assert!(user_type.fields.contains_key("updated_name"));
        assert!(user_type.fields.contains_key("new_name"));

        assert!(!user_type.fields.contains_key("old_name"));
    }

    #[test]
    fn test_rename_field_not_found() {
        let sdl = r#"
            type User {
                name: String
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let mut mappings = IndexMap::new();
        mappings.insert(
            "non_existent_field".to_string(),
            FieldInfo::new(vec!["new_name".to_string()], TypeName::new("User")),
        );

        let actual = RenameFields::new(mappings).transform(config).to_result();

        assert!(actual.is_err());
        let expected_error = ValidationError::new(
            "Field 'non_existent_field' not found in type 'User'.".to_string(),
        );

        assert_eq!(actual.unwrap_err(), expected_error);
    }

    #[test]
    fn test_rename_type_not_found() {
        let sdl = r#"
            type User {
                name: String
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let mut mappings = IndexMap::new();
        mappings.insert(
            "name".to_string(),
            FieldInfo::new(
                vec!["new_name".to_string()],
                TypeName::new("NonExistentType"),
            ),
        );
        let actual = RenameFields::new(mappings).transform(config).to_result();

        assert!(actual.is_err());

        let expected_error = ValidationError::new("Type 'NonExistentType' not found.".into());

        assert_eq!(actual.unwrap_err(), expected_error);
    }

    #[test]
    fn test_rename_all_suggestions_in_use() {
        let sdl = r#"
            type User {
                old_name: String
                new_name: String
                updated_name: String
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let mut mappings = IndexMap::new();
        mappings.insert(
            "old_name".to_string(),
            FieldInfo::new(
                vec!["new_name".to_string(), "updated_name".to_string()],
                TypeName::new("User"),
            ),
        );

        let actual = RenameFields::new(mappings).transform(config).to_result();
        let expected_error = ValidationError::new(
            "Could not rename field 'old_name' in type 'User'. All suggested names are already in use."
                .into(),
        );

        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err(), expected_error);
    }
}
