use std::path::PathBuf;

use crate::blueprint::*;
use crate::config::{Config, Field};
use crate::lambda::{Expression, Lambda};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};
use crate::config;

pub fn compile_file(
    _config: &config::Config,
    _field: &config::Field,
    file: &config::File,
) -> Valid<Expression, String> {
    Valid::from_option(
        file.src.as_ref(),
        "No file src defined".to_string(),
    )
        .and_then(|src| {
            let path = PathBuf::try_from(src)
                .map_err(|e| ValidationError::new(e.to_string()));

            match path {
                Ok(x) => {
                    if x.exists() {
                        let ext = x.extension()
                            .map(|x| x.to_string_lossy().to_string())
                            .unwrap_or("".to_string());

                        if ext != "yml" && ext != "yaml" && ext != "json" {
                            Err(ValidationError::new(format!("Extension {:?} not supported.", ext))).into()
                        } else {
                            Ok(x).into()
                        }
                    } else {
                        Err(ValidationError::new(format!("File {:?} not found.", x))).into()
                    }
                },
                Err(e) => Err(e).into()
            }
        })
        .map(|path| Lambda::from_file(path).expression)
}

pub fn update_file<'a>(
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
    TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
        |(config, field, type_of, _), b_field| {
            let Some(file) = &field.file else {
                return Valid::succeed(b_field);
            };

            compile_file(config, field, file)
                .map(|resolver| b_field.resolver(Some(resolver)))
                .and_then(|b_field| b_field.validate_field(type_of, config).map_to(b_field))
        },
    )
}
