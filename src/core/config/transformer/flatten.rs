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
            let mut unused = vec![];
            if let Some((field, ty_of)) = iter(IterHelper::new(&config, &root), &mut unused) {
                let ty = config.types.get_mut(&root).unwrap();
                let fields = &mut ty.fields;
                fields.get_mut(field.as_str()).unwrap().type_of = ty_of;
                for unused in unused {
                    config.types.remove(&unused);
                }
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
fn iter(iter_helper: IterHelper, unused: &mut Vec<String>) -> Option<(String, String)> {
    let config = iter_helper.config;
    let ty = iter_helper.ty;
    let ty = config.types.get(ty).unwrap();

    if ty.fields.len() == 1 {
        if let Some((field_name, field)) = ty.fields.iter().next() {
            let ty_of = field.type_of.clone();

            if SCALAR_TYPES.contains(ty_of.as_str()) {
                return if let Some(field_name) = iter_helper.root_field {
                    Some((field_name, ty_of))
                } else {
                    Some((field_name.clone(), ty_of))
                };
            }
            unused.push(ty_of.as_str().to_string());
            return if iter_helper.root_field.is_some() {
                iter(iter_helper.ty(ty_of.as_str()), unused)
            } else {
                iter(
                    iter_helper
                        .ty(ty_of.as_str())
                        .root_field(Some(field_name.clone())),
                    unused,
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
    fn test_flatten() {
        let config = Config::from_sdl(
            std::fs::read_to_string(tailcall_fixtures::generator::FLATTEN)
                .unwrap()
                .as_str(),
        )
        .to_result()
        .unwrap();
        let transformed = Flatten.transform(config).to_result().unwrap();
        insta::assert_snapshot!(transformed.to_sdl())
    }
}
