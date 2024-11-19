use derive_setters::Setters;
use tailcall_valid::{ValidationError, Validator};

use crate::core::blueprint::telemetry::{to_opentelemetry, Telemetry};
use crate::core::blueprint::{Blueprint, Definition, Server, Upstream};
use crate::core::config::{Batch, ConfigModule};
use crate::core::ir::model::{IO, IR};

#[derive(Clone, Debug, Default, Setters)]
pub struct RuntimeConfig {
    pub server: Server,
    pub upstream: Upstream,
    pub telemetry: Telemetry,
}

impl TryFrom<&ConfigModule> for RuntimeConfig {
    type Error = ValidationError<String>;

    fn try_from(value: &ConfigModule) -> Result<Self, Self::Error> {
        let telemetry = to_opentelemetry()
            .transform::<Option<Telemetry>>(|a, _| Some(a), |c| c.unwrap_or_default())
            .try_fold(value, None)
            .to_result()?
            .ok_or(ValidationError::new(
                "Telemetry not set correctly".to_string(),
            ))?;

        let runtime_config = Self {
            server: Server::try_from(value.clone())?,
            upstream: Upstream::try_from(value)?,
            telemetry,
        };

        let blueprint = &Blueprint::try_from(value)?;

        Ok(apply_batching(runtime_config, blueprint))
    }
}

// Apply batching if any of the fields have a @http directive with groupBy field
pub fn apply_batching(mut runtime_config: RuntimeConfig, blueprint: &Blueprint) -> RuntimeConfig {
    for def in blueprint.definitions.iter() {
        if let Definition::Object(object_type_definition) = def {
            for field in object_type_definition.fields.iter() {
                if let Some(IR::IO(IO::Http { group_by: Some(_), .. })) = field.resolver.clone() {
                    runtime_config.upstream.batch =
                        runtime_config.upstream.batch.or(Some(Batch::default()));
                    return runtime_config;
                }
            }
        }
    }
    runtime_config
}
