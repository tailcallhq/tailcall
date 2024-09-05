use crate::core::config::Config;
use crate::core::valid::{Valid, Validator};
use crate::core::Transform;

#[derive(Clone)]
pub struct Location {
    pub new_field_name: String,
    pub type_name: String,
}

#[derive(Clone)]
pub struct RenameFields(Vec<(String, Location)>);

impl RenameFields {
    pub fn new(mappings: Vec<(String, Location)>) -> Self {
        Self(mappings)
    }
}

impl Transform for RenameFields {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut value: Self::Value) -> Valid<Self::Value, Self::Error> {
        Valid::from_iter(self.0.iter(), |(field_name, field_info)| {
            let type_name = field_info.type_name.as_str();
            let new_field_name = field_info.new_field_name.as_str();

            if let Some(type_def) = value.types.get_mut(type_name) {
                if type_def.fields.contains_key(new_field_name) {
                    return Valid::fail(format!(
                        "Field '{}' already exists in type '{}'.",
                        new_field_name, type_name
                    ));
                } else {
                    if let Some(field_def) = type_def.fields.remove(field_name) {
                        type_def.fields.insert(new_field_name.to_owned(), field_def);
                        Valid::succeed(())
                    } else {
                        Valid::fail(format!(
                            "Field '{}' not found in type '{}'.",
                            field_name, type_name
                        ))
                    }
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
    fn test_rename_field() {
        let sdl = r#"
            type User {
                old_name: String
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let mappings = vec![(
            "old_name".to_string(),
            Location { new_field_name: "new_name".into(), type_name: "User".into() },
        )];

        let transformed_config = RenameFields::new(mappings)
            .transform(config)
            .to_result()
            .unwrap();

        let user_type = transformed_config.find_type("User").unwrap();

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

        let mappings = vec![(
            "non_existent_field".into(),
            Location { new_field_name: "new_name".into(), type_name: "User".into() },
        )];

        let actual = RenameFields::new(mappings).transform(config).to_result();

        let expected_error = ValidationError::new(
            "Field 'non_existent_field' not found in type 'User'.".to_string(),
        );

        assert!(actual.is_err());
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

        let mappings = vec![(
            "name".into(),
            Location {
                new_field_name: "new_name".into(),
                type_name: "NonExistentType".into(),
            },
        )];

        let actual = RenameFields::new(mappings).transform(config).to_result();
        let expected_error = ValidationError::new("Type 'NonExistentType' not found.".into());

        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err(), expected_error);
    }

    #[test]
    fn test_duplicate_rename_type() {
        let sdl = r#"
            type User {
                name: String
                newName: String
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let mappings = vec![(
            "name".into(),
            Location {
                new_field_name: "newName".into(),
                type_name: "User".into(),
            },
        )];

        let actual = RenameFields::new(mappings).transform(config).to_result();
        let expected_error =
            ValidationError::new("Field 'newName' already exists in type 'User'.".into());

        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err(), expected_error);
    }
}
