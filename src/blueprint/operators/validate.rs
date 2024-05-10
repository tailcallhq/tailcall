use crate::blueprint::*;
use crate::config::{self, ConfigModule, Field};
use core::time::Duration;
use crate::lambda::{Expression, IO};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};
use crate::cli::javascript::Runtime;
// use crate::cli::javascript::call;
// use crate::cli::javascript::{Event, Command, setup_builtins};
// use opentelemetry::trace::FutureExt;
// use rquickjs::{Context, Runtime};
// use rquickjs;
// use reqwest::Request;


// pub fn compile_validate(
//     config_module: &ConfigModule,
//     field: &Field,
//     value: &serde_json::Value,
//     validate: bool,
// ) -> Valid<Expression, String> {
//     Valid::fail("The validation function does not return a boolean".to_string())
//         .when(
//             call()
//         )
// }


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

            let script = Script {
                source: config_module.extensions.script.clone().expect("script is required"),
                timeout: config_module.server.script.clone().map_or_else(|| None, |script| script.timeout).map(Duration::from_millis),
            };
            let js_func = &field.validate.unwrap().js;
            let runtime = Runtime::new(script);
            let value = &field.const_field;
            todo!()

            Valid::succeed(b_field)
        }
    )
}