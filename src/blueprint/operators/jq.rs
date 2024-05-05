use crate::blueprint::*;
use crate::config;
use crate::config::Field;
use crate::lambda::Expression;
use crate::lambda::Expression::Jq;
use crate::try_fold::TryFold;
use crate::valid::{Valid, Validator};

pub struct CompileExpr<'a> {
    pub query: &'a str,
}

pub fn compile_jq(inputs: CompileExpr) -> Valid<Expression, String> {
    let mut defs = jaq_interpret::ParseCtx::new(vec![]);
    defs.insert_natives(jaq_core::core());
    defs.insert_defs(jaq_std::std());

    let filter = inputs.query;
    let (filter, errs) = jaq_parse::parse(filter, jaq_parse::main());
    let errs = errs
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    if !errs.is_empty() {
        return Valid::fail(errs);
    }

    Valid::from_option(filter, errs)
        .map(|v| defs.compile(v))
        .map(Jq)
}

pub fn update_jq_field<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        |(_config_module, field, _, _), b_field| {
            let Some(const_field) = &field.jq else {
                return Valid::succeed(b_field);
            };

            compile_jq(CompileExpr { query: &const_field.query })
                .map(|resolver| b_field.resolver(Some(resolver)))
        },
    )
}
