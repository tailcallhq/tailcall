use derive_setters::Setters;

use crate::core::config::Config;
use crate::core::scalar::SCALAR_TYPES;
use crate::core::valid::{Valid, Validator};
use crate::core::Transform;

#[derive(Default)]
pub struct Flatten;

#[derive(Setters)]
struct IterHelper<'a> {
    config: &'a Config,
    ty: &'a str,
}

impl<'a> IterHelper<'a> {
    fn new(config: &'a Config, ty: &'a str) -> Self {
        Self { config, ty }
    }
}

impl Transform for Flatten {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        // used to store all unused types
        let mut unused_collect = vec![];

        // iterate over Query, Mutation, Sub
        Valid::from_iter(config.types.clone(), |(ty_name, ty)| {
            Valid::from_iter(ty.fields, |(name, field)| {
                let mut unused = vec![];
                let ty = &field.type_of;
                if !SCALAR_TYPES.contains(ty.as_str()) {
                    unused.push(ty.as_str().to_string());
                }
                if let Some(ty_of) = iter(IterHelper::new(&config, ty), &mut unused) {
                    let ty = config.types.get_mut(&ty_name).unwrap();
                    let fields = &mut ty.fields;
                    fields.get_mut(name.as_str()).unwrap().type_of = ty_of;
                    unused_collect.extend(unused);
                }
                Valid::succeed(())
            })
        })
        .and_then(|_| {
            for unused in unused_collect {
                config.types.remove(&unused);
            }
            Valid::succeed(config)
        })
    }
}

#[inline(always)]
fn iter(iter_helper: IterHelper, unused: &mut Vec<String>) -> Option<String> {
    let config = iter_helper.config;
    let ty = iter_helper.ty;
    let ty = config.types.get(ty)?;

    if ty.fields.len() == 1 {
        if let Some((_, field)) = ty.fields.iter().next() {
            let ty_of = field.type_of.clone();

            if SCALAR_TYPES.contains(ty_of.as_str()) {
                return Some(ty_of);
            }
            unused.push(ty_of.as_str().to_string());
            return iter(iter_helper.ty(ty_of.as_str()), unused);
        }
    } else {
        return unused.pop();
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
        // insta::assert_snapshot!(transformed.to_sdl());
        println!("{}", transformed.to_sdl());
    }

    #[test]
    fn test_flatten_complex() {
        let config = Config::from_sdl(
            std::fs::read_to_string(tailcall_fixtures::generator::FLATTEN_COMPLEX)
                .unwrap()
                .as_str(),
        )
        .to_result()
        .unwrap();
        let transformed = Flatten.transform(config).to_result().unwrap();
        insta::assert_snapshot!(transformed.to_sdl());
    }
}
