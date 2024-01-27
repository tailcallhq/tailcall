use std::fs::File;
use std::io::Read as _;
use std::path::PathBuf;

use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::lambda::Expression;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub struct CompileFile<'a> {
    pub config: &'a config::Config,
    pub field: &'a config::Field,
    pub file: &'a crate::config::File,
    pub validate: bool,
}

pub fn compile_file(inputs: CompileFile<'_>) -> Valid<Expression, String> {
    let Some(path) = &inputs.file.src else {
        return Valid::fail("@file must have parameter src".to_string());
    };

    let path = PathBuf::from(path);

    let ext = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_string())
        .unwrap_or("".to_string());

    if !path.is_file() {
        return Valid::fail(format!("file not found: {:?}", path));
    }

    let Ok(mut file) = File::open(&path) else {
        return Valid::fail(format!("failed to open file: {:?}", path));
    };

    let mut data: String = String::new();
    match file.read_to_string(&mut data) {
        Ok(_) => {}
        Err(e) => return Valid::fail(format!("failed to read file: {}", e)),
    };

    let value = if ext == "json" {
        match serde_json::from_str::<serde_json::Value>(&data) {
            Ok(x) => x,
            Err(e) => return Valid::fail(format!("failed to parse JSON file: {}", e)),
        }
    } else if ext == "yaml" || ext == "yml" {
        match serde_yaml::from_str::<serde_json::Value>(&data) {
            Ok(x) => x,
            Err(e) => return Valid::fail(format!("failed to parse YAML file: {}", e)),
        }
    } else {
        return Valid::fail(format!("file extension {:?} not supported", ext));
    };

    compile_const(CompileConst {
        config: inputs.config,
        field: inputs.field,
        value: &value,
        validate: inputs.validate,
    })
}

pub fn update_file<'a>(
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
    TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(
        |(config, field, _, _), b_field| {
            let Some(file) = &field.file else {
                return Valid::succeed(b_field);
            };

            compile_file(CompileFile { config, field, file, validate: true })
                .map(|resolver| b_field.resolver(Some(resolver)))
        },
    )
}
