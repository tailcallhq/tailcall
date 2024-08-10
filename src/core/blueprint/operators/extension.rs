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
                let params = match DynamicValue::try_from(&extension.params) {
                    Ok(params) => params,
                    Err(_) => {
                        return Valid::fail(format!(
                            "Could not prepare dynamic value for `{}`",
                            name
                        ))
                    }
                };
                let plugin = match config_module
                    .extensions()
                    .plugin_extensions
                    .get(&extension.name)
                {
                    Some(plugin) => plugin.clone(),
                    None => {
                        return Valid::fail(format!(
                            "Could not find extension `{}` for `{}`",
                            extension.name, name
                        ))
                    }
                }
                .clone();
                plugin.load();
                let extension_resolver = IR::Extension {
                    plugin,
                    params,
                    ir: Box::new(
                        b_field
                            .resolver
                            .unwrap_or(IR::ContextPath(vec![b_field.name.clone()])),
                    ),
                };
                b_field.resolver = Some(extension_resolver);
            }
            Valid::succeed(b_field)
        },
    )
}

pub trait ExtensionLoader: std::fmt::Debug + Send + Sync {
    fn load(&self) {}

    fn modify_inner(&self, ir: Box<IR>) -> Box<IR> {
        ir
    }

    fn prepare(&self, ir: Box<IR>, params: ConstValue) -> Box<IR>;

    fn process(&self, params: ConstValue, value: ConstValue) -> Result<ConstValue, Error>;
}
