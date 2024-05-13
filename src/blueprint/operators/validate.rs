use crate::blueprint::*;
use crate::config::{self, ConfigModule, Field};
use core::time::Duration;
use crate::lambda::{Expression, IO};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};
use rquickjs::{Runtime, Context, Function, FromJs};
use crate::cli::javascript::setup_builtins;


pub fn update_validate<'a>(
    type_name: &'a str,
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
        |(config_module, field, _type, _), mut b_field| {
            if field.validate.is_some()
                || _type.validate.is_some()
                || config_module
                    .find_type(&field.type_of)
                    .and_then(|_type| _type.validate.as_ref())
                    .is_some()
            {
                if !config_module.is_scalar(type_name) {
                    return Valid::fail("@validate can only be used on custom scalars".to_owned());
                }
            }


            let js_func = &field.validate.as_ref().unwrap().js;
            let value = &field.const_field.as_ref().unwrap().body;
            let script = config_module.extensions.script.clone().expect("script is required");
            let js_runtime = Runtime::new().ok().unwrap();
            let context = Context::full(&js_runtime).ok().unwrap();

            // context.with(|ctx| {
            //     setup_builtins(&ctx).ok().unwrap();
            //     ctx.eval(script)
            // });

            context.with(|ctx| {
                let fn_as_value = ctx
                    .globals()
                    .get::<&str, Function>(js_func.as_str())
                    .map_err(|_| anyhow::anyhow!("globalThis not initialized"));

                let function = fn_as_value.ok().unwrap()
                    .as_function()
                    .ok_or(anyhow::anyhow!("`{js_func}` is not a function"));

            });

            todo!();

            Valid::succeed(b_field)
        }
    )
}