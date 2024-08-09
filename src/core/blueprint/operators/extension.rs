use async_graphql_value::ConstValue;

use crate::core::blueprint::*;
use crate::core::config;
use crate::core::config::Field;
use crate::core::ir::model::IR;
use crate::core::ir::Error;
use crate::core::try_fold::TryFold;
use crate::core::valid::Valid;

pub fn update_extension<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
        |(config_module, field, _type_of, name), mut b_field| {
            if let Some(extension) = &field.extension {
                let params = DynamicValue::try_from(&extension.params)
                    .unwrap_or_else(|_| panic!("Could not prepare dynamic value for `{}`", name));
                let plugin = config_module
                    .extensions()
                    .plugin_extensions
                    .get(&extension.name)
                    .unwrap_or_else(|| {
                        panic!(
                            "Could not find extension `{}` for `{}`",
                            extension.name, name
                        )
                    })
                    .clone();
                plugin.load();
                let extension_resolver =
                    IR::Extension { plugin, params, ir: b_field.resolver.map(Box::new) };
                b_field.resolver = Some(extension_resolver);
            }
            Valid::succeed(b_field)
        },
    )
}

pub trait ExtensionLoader: std::fmt::Debug + Send + Sync {
    // TODO: signature is not the desired one
    fn load(&self);

    // TODO: signature is not the desired one
    fn prepare(&self, ir: Box<IR>, params: ConstValue) -> Box<IR>;

    // TODO: signature is not the desired one
    fn process(&self, params: ConstValue, value: ConstValue) -> Result<ConstValue, Error>;
}

// TODO: remove unused code
// impl<Json> DynamicExtension<Json>
// where
// Json: JsonLikeOwned {
//     fn new(init: Box<dyn Fn(&DynamicExtension<Json>)>, prepare: Box<dyn
// Fn(&DynamicExtension<Json>, IR, &[Json]) -> IR>, process: Box<dyn
// Fn(&DynamicExtension<Json>, &[Json], Json) -> Json>) -> Self {
//         DynamicExtension { init, prepare, process }
//     }

//     fn call_init(&self) {
//         (self.init)(self);
//     }

//     fn call_prepare(&self, ir: IR, params: &[Json]) -> IR {
//         (self.prepare)(self, ir, params)
//     }

//     fn call_process(&self, params: &[Json], value: Json) -> Json {
//         (self.process)(self, params, value)
//     }
// }
