use crate::blueprint::*;
use crate::config::{self, ConfigModule, Field};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};
use rquickjs::{Runtime, Context, Function, FromJs};
use serde_json;
use crate::cli::javascript::setup_builtins;


fn json_to_js<'js>(value: serde_json::Value, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<(rquickjs::Value<'js>,)> {
    let object = rquickjs::Object::new(ctx.clone())?;

    match value {
        serde_json::Value::Number(num) => {
            if num.is_i64() {
                object.set("value", num.as_i64().unwrap())?;
            } else if num.is_f64() {
                object.set("value", num.as_f64().unwrap())?;
            }
        }
        serde_json::Value::String(string) => {
            object.set("value", string)?;
        }
        serde_json::Value::Bool(boolean) => {
            object.set("value", boolean)?;
        }
        serde_json::Value::Object(obj) => {
            for (key, value) in obj {
                let (value,) = json_to_js(value, ctx)?;
                object.set(key, value)?;
            }
        }
        _ => {}
    }

    Ok((object.into_value(),))
}


fn call(func: String, val: serde_json::Value, src: String) -> () {
    let js_runtime = Runtime::new().ok().unwrap();
    let context = Context::full(&js_runtime).ok().unwrap();

    // context.with(|ctx| {
    //     setup_builtins(&ctx).ok().unwrap();
    //     ctx.eval(src)
    // });

    context.with(|ctx| {
        let fn_as_value = ctx
            .globals()
            .get::<&str, Function>(func.as_str())
            .map_err(|_| anyhow::anyhow!("globalThis not initialized"))
            .ok().unwrap();

        let function = fn_as_value
            .as_function()
            .ok_or(anyhow::anyhow!("`{func}` is not a function"))
            .ok().unwrap();

        let custom_scalar = json_to_js(val.clone(), &ctx).ok().unwrap();
        let command: Option<rquickjs::Value> = function.call(custom_scalar).ok();

        ()
    });
}

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

            todo!();

            Valid::succeed(b_field)
        }
    )
}