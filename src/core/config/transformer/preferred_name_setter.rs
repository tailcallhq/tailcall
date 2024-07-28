use std::collections::{HashMap, HashSet};

use inflector::Inflector;

use crate::core::config::Config;
use crate::core::valid::Valid;
use crate::core::Transform;

// goes through operation type names and set's it's output type name from
// suggested names;
pub struct PreferredNameSetter(HashSet<String>);

impl PreferredNameSetter {
    pub fn new(suggested_names: HashSet<String>) -> Self {
        Self(suggested_names)
    }

    pub fn apply(&self, mut config: Config) -> Config {
        let _ty_names = vec![
            config.schema.query.as_ref(),
            config.schema.mutation.as_ref(),
            config.schema.subscription.as_ref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let mut finalized_candidates = HashMap::new();
        for type_name in _ty_names {
            if let Some(type_1) = config.types.get(type_name) {
                for (field_name, field_1) in type_1.fields.iter() {
                    let singularized_name = field_name.to_singular().to_pascal_case();
                    if config.types.contains_key(&singularized_name)
                        || finalized_candidates.contains_key(&field_1.type_of)
                        || config.is_scalar(&field_1.type_of)
                        || !self.0.contains(field_name)
                    {
                        continue;
                    }
                    finalized_candidates.insert(field_1.type_of.to_owned(), singularized_name);
                }
            }
        }

        for (old_type_name, new_type_name) in finalized_candidates {
            if let Some(type_) = config.types.remove(old_type_name.as_str()) {
                // Add newly generated type.
                config.types.insert(new_type_name.to_owned(), type_);

                // Replace all the instances of old name in config.
                for actual_type in config.types.values_mut() {
                    for actual_field in actual_type.fields.values_mut() {
                        if actual_field.type_of == old_type_name {
                            // Update the field's type with the new name
                            actual_field.type_of.clone_from(&new_type_name);
                        }
                    }
                }
            }
        }

        config
    }
}

impl Transform for PreferredNameSetter {
    type Value = Config;
    type Error = String;

    fn transform(&self, value: Self::Value) -> crate::core::valid::Valid<Self::Value, Self::Error> {
        Valid::succeed(self.apply(value))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::fs;

    use tailcall_fixtures::configs;

    use crate::core::config::transformer::PreferredNameSetter;
    use crate::core::config::Config;
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    fn read_fixture(path: &str) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn test_should_use_user_suggested_name() {
        let config = Config::from_sdl(read_fixture(configs::CONFLICTING_TYPE_NAMES).as_str())
            .to_result()
            .unwrap();

        let mut suggested_names = HashSet::new();
        suggested_names.insert("post".to_owned());
        suggested_names.insert("users".to_owned());
        suggested_names.insert("todos".to_owned());

        // transformer without suggested names, should modify the config with user
        // suggested names.
        let transformed_config = PreferredNameSetter::new(suggested_names)
            .transform(config.clone())
            .to_result()
            .unwrap();
        insta::assert_snapshot!("with_suggested_names", transformed_config.to_sdl());

        // transformer without suggested names, should not modify the config.
        let transformed_config = PreferredNameSetter::new(HashSet::new())
            .transform(config)
            .to_result()
            .unwrap();
        insta::assert_snapshot!("without_suggested_names", transformed_config.to_sdl());
    }
}
