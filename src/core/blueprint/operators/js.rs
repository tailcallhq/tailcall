use crate::core::blueprint::FieldDefinition;
use crate::core::config;
use crate::core::config::Field;
use crate::core::lambda::IO;
use crate::core::try_fold::TryFold;
use crate::core::valid::Valid;
use crate::{ConfigModule, Expression, ValidationError, Validator};

pub struct CompileJs<'a> {
    pub name: &'a str,
    pub script: &'a Option<String>,
}

pub fn compile_js(inputs: CompileJs) -> Valid<Expression, String> {
    #[cfg(not(feature = "js"))]
    return js_disabled(inputs);

    #[cfg(feature = "js")]
    js_enabled(inputs)
}

#[cfg(not(feature = "js"))]
fn js_disabled(_: CompileJs) -> Valid<Expression, String> {
    Valid::fail("js is disabled or unsupported on this platform".to_string())
}

#[cfg(feature = "js")]
fn js_enabled(inputs: CompileJs) -> Valid<Expression, String> {
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
            .map(|_| Expression::IO(IO::Js { name: name.to_string() }))
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
