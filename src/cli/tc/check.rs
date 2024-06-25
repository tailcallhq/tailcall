use super::helpers::{log_endpoint_set, display_schema};
use super::cli::fmt::Fmt;
use crate::core::blueprint::Blueprint;
use crate::core::config::reader::ConfigReader;
use crate::cli::CLIError;
use crate::core::config::Source;
use crate::core::runtime::TargetRuntime;
use anyhow::Result;

pub(super) async fn check_command(
    file_paths: Vec<String>,
    n_plus_one_queries: bool,
    schema: bool,
    format: Option<Source>,
    config_reader: &ConfigReader,
    runtime: TargetRuntime,
) -> Result<()> {
    let config_module = (config_reader.read_all(&file_paths)).await?;
    log_endpoint_set(&config_module.extensions.endpoint_set);
    if let Some(format) = format {
        Fmt::display(format.encode(&config_module)?);
    }
    let blueprint = Blueprint::try_from(&config_module).map_err(CLIError::from);

    match blueprint {
        Ok(blueprint) => {
            tracing::info!("Config {} ... ok", file_paths.join(", "));
            Fmt::log_n_plus_one(n_plus_one_queries, &config_module.config);
            // Check the endpoints' schema
            let _ = config_module
                .extensions
                .endpoint_set
                .into_checked(&blueprint, runtime)
                .await?;
            if schema {
                display_schema(&blueprint);
            }

            Ok(())
        }
        Err(e) => Err(e.into()),
    }

}

