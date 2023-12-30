use crate::try_fold::TryFold;
use crate::valid::Valid;
use crate::{blueprint::*, config};
use crate::config::{Config, Field, ExprBody};

pub fn update_expr<'a>() -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
    TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(config, field, ty, name), b_field| {
        let Some(expr) = &field.expr else {
            return Valid::succeed(b_field);
        };

        match &expr.body {
            ExprBody::Http(http) => {
                let field_with_http = (*field).clone().http(http.clone());
                let http_field_def = update_http()
                    .try_fold(&(config, &field_with_http, ty, name), b_field);
                http_field_def
            },
            _ => Valid::fail(format!("invalid expr: unsupported operator in body"))
        }
    })
}
