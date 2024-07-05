use crate::core::config::AddField;
use crate::core::config::Config;
use crate::core::config::Omit;
use crate::core::config::Type;
use crate::core::transform::Transform;
use crate::core::valid::Valid;
use crate::core::valid::Validator;

/// Flat single field type and inline to Query directly by addField
#[derive(Default)]
pub struct FlattenSingleField;

fn get_single_field_path(config: &Config, field_name: &str, type_name: &str) -> Option<Vec<String>> {
    let mut path = Vec::new();
    path.push(field_name.to_owned());
    if config.is_scalar(type_name) || config.enums.contains_key(type_name) {
        return Some(path);
    }
    let ty = config.types.get(type_name);
    if let Some(ty) = ty {
        if ty.fields.len() == 1 {
            if let Some((sub_field_name, sub_field)) = ty.fields.first_key_value() {
                let sub_path = get_single_field_path(&config, &sub_field_name, &sub_field.type_of);
                sub_path.map(|sub_path| path.extend(sub_path));
                Some(path)
            } else {
                None
            }
        } else {
            None
        }

    } else {
        None
    }
}

impl Transform for FlattenSingleField {
    type Value = Config;
    type Error = String;
    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        let origin_config = config.clone();
        if let Some(root) = &config.schema.query {
            let root_query = config.types.get_mut(root);
            if let Some(root_query) = root_query {
                let field_trans = Valid::from_iter(root_query.fields.iter_mut(), |(name, field)| {
                    if let Some(path) = get_single_field_path(&origin_config, &name, &field.type_of) {
                        if path.len() > 1 {
                            field.omit = Some(Omit{});
                            root_query.added_fields.push(AddField {
                                name: name.to_owned(),
                                path
                            });
                        }
                    }
                    Valid::succeed(())
                });
                field_trans.map(|_| config)
            } else {
                Valid::fail("Query type is not existed.".to_owned())
            }
        } else {
            Valid::succeed(config)
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use tailcall_fixtures::configs;

    use super::FlattenSingleField;
    use crate::core::config::Config;
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    fn read_fixture(path: &str) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn test_type_name_generator_transform() {
        let config = Config::from_sdl(read_fixture(configs::FLATTEN_SINGLE_FIELD).as_str())
            .to_result()
            .unwrap();

        let transformed_config = FlattenSingleField.transform(config).to_result().unwrap();
        insta::assert_snapshot!(transformed_config.to_sdl());
    }

}
