use std::sync::Arc;

use crate::blueprint::*;
use crate::config;
use crate::config::Field;
use crate::javascript::Runtime;
use crate::lambda::Expression;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};

pub struct CompileJs<'a> {
    pub name: &'a str,
    pub script: &'a Option<String>,
}

pub fn compile_js(inputs: CompileJs) -> Valid<Expression, String> {
    let name = inputs.name;
    let quickjs = rquickjs::Runtime::new().unwrap();
    let ctx = rquickjs::Context::full(&quickjs).unwrap();

    Valid::from_option(inputs.script.as_ref(), "script is required".to_string()).and_then(
        |script| {
            Valid::from(
                ctx.with(|ctx| {
                    ctx.eval::<(), &str>(script)?;
                    Ok::<_, anyhow::Error>(())
                })
                .map_err(|e| ValidationError::new(e.to_string())),
            )
            .map(|_| {
                Expression::Js(Arc::new(Runtime::new(
                    name.to_string(),
                    Script { source: script.clone(), timeout: None },
                )))
            })
        },
    )
}

pub fn update_js_field<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        |(module, field, _, _), b_field| {
            let Some(js) = &field.script else {
                return Valid::succeed(b_field);
            };

            compile_js(CompileJs { script: &module.extensions.script, name: &js.name })
                .map(|resolver| b_field.resolver(Some(resolver)))
        },
    )
}
