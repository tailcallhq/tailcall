use derive_setters::Setters;

use crate::core::config::Config;
use crate::core::scalar::SCALAR_TYPES;
use crate::core::valid::Valid;
use crate::core::Transform;

#[derive(Default)]
pub struct Flatten;

#[derive(Setters)]
struct IterHelper<'a> {
    config: &'a Config,
    ty: &'a str,
    root_field: Option<String>,
}

impl<'a> IterHelper<'a> {
    fn new(config: &'a Config, ty: &'a str) -> Self {
        Self { config, ty, root_field: None }
    }
}

impl Transform for Flatten {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        for root in roots(&config) {
            if let Some((field, ty_of)) = iter(IterHelper::new(&config, &root)) {
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
fn iter(iter_helper: IterHelper) -> Option<(String, String)> {
    let config = iter_helper.config;
    let ty = iter_helper.ty;
    let ty = config.types.get(ty).unwrap();

    if ty.fields.len() == 1 {
        if let Some((field_name, field)) = ty.fields.iter().next() {
            if SCALAR_TYPES.contains(field.type_of.as_str()) {
                return if let Some(field_name) = iter_helper.root_field {
                    Some((field_name, field.type_of.clone()))
                } else {
                    Some((field_name.clone(), field.type_of.clone()))
                };
            }
            return if iter_helper.root_field.is_some() {
                iter(iter_helper.ty(field.type_of.as_str()))
            } else {
                iter(
                    iter_helper
                        .ty(field.type_of.as_str())
                        .root_field(Some(field_name.clone())),
                )
            };
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::Flatten;
    use crate::core::config::Config;
    use crate::core::valid::Validator;
    use crate::core::Transform;

    #[test]
    fn test_foo() {
        let config = Config::from_sdl(
            std::fs::read_to_string(tailcall_fixtures::generator::FLATTEN)
                .unwrap()
                .as_str(),
        )
        .to_result()
        .unwrap();
        let transformed = Flatten.transform(config).to_result().unwrap();
        println!("{}", transformed.to_sdl());
    }
}
