use crate::core::config::Config;
use crate::core::scalar::SCALAR_TYPES;
use crate::core::Transform;
use crate::core::valid::Valid;

#[derive(Default)]
pub struct Flatten;

impl Transform for Flatten {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        for root in roots(&config) {
            println!("{}", root);
            if let Some((field, ty_of)) = iter(&config, &root, None) {
                let ty = config.types.get_mut(&root).unwrap();
                let fields = &mut ty.fields;
                fields.get_mut(field.as_str()).unwrap().type_of = ty_of;
            }
        }
        /*Valid::from_iter(roots(&config), |root| {
            Valid::from_option(config.types.get(&root), "Root type not found")
        }).and_then(|root_types| {
            Valid::from_iter(root_types, |ty| {
                if let Some((field, ty_of)) = iter(&config, ty, None) {

                }
            })
        })*/
        Valid::succeed(config)
    }
}

fn roots(config: &Config) -> Vec<String> {
    let schema = config.schema.clone();
    let mut root_types = vec![];
    if let Some(query) = schema.query {
        root_types.push(query);
    }
    if let Some(mutation) = schema.mutation {
        root_types.push(mutation);
    }
    if let Some(subscription) = schema.subscription {
        root_types.push(subscription);
    }
    root_types
}

#[inline(always)]
fn iter(config: &Config, ty: &str, root_field: Option<String>) -> Option<(String, String)> {
    let ty = config.types.get(ty).unwrap();

    if ty.fields.len() == 1 {
        for (field_name, field) in ty.fields.iter() {
            if SCALAR_TYPES.contains(field.type_of.as_str()) {
                return if let Some(field_name) = root_field {
                    Some((field_name, field.type_of.clone()))
                } else {
                    Some((field_name.clone(), field.type_of.clone()))
                }
            }
            return if let Some(field_name) = root_field {
                iter(config, &field.type_of, Some(field_name))
            }else {
                iter(config, &field.type_of, Some(field_name.clone()))
            };
        }
    }
    None
}

#[cfg(test)]
mod test {
    use crate::core::config::Config;
    use crate::core::Transform;
    use crate::core::valid::Validator;

    use super::Flatten;

    const SDL: &str = r#"
        schema {
          query: Query
        }

        type Query {
          foo: Foo
        }

        # Type with only one field
        type Foo {
          bar: Bar
        }

        # Type with only one field
        type Bar {
          a: Int
        }
    "#;

    #[test]
    fn test_foo() {
        let config = Config::from_sdl(SDL).to_result().unwrap();
        let transformed = Flatten::default().transform(config).to_result().unwrap();
        println!("{}", transformed.to_sdl());
    }
}