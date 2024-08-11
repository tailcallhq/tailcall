use std::io::Write;

use anyhow::Result;

use super::helpers::{display_schema, log_endpoint_set};
use crate::cli::fmt::Fmt;
use crate::cli::CLIError;
use crate::core::blueprint::Blueprint;
use crate::core::config::reader::ConfigReader;
use crate::core::config::Source;
use crate::core::runtime::TargetRuntime;

pub struct CheckParams {
    pub file_paths: Vec<String>,
    pub n_plus_one_queries: bool,
    pub schema: bool,
    pub format: Option<Source>,
    pub runtime: TargetRuntime,
}

pub async fn check_command(
    params: CheckParams,
    config_reader: &ConfigReader,
    mut write_buf: Option<&mut dyn Write>,
) -> Result<()> {
    let CheckParams { file_paths, n_plus_one_queries, schema, format, runtime } = params;

    let config_module = (config_reader.read_all(&file_paths)).await?;
    log_endpoint_set(&config_module.extensions().endpoint_set);
    if let Some(format) = format {
        let format_msg = format.encode(&config_module)?;
        if let Some(write_buf) = &mut write_buf {
            writeln!(write_buf, "{}", format_msg)?;
        }
        Fmt::display(format_msg);
    }
    let blueprint = Blueprint::try_from(&config_module).map_err(CLIError::from);

    match blueprint {
        Ok(blueprint) => {
            let config_msg = format!("Config {} ... ok", file_paths.join(", "));
            let n_plus_message =
                Fmt::format_n_plus_one_message(n_plus_one_queries, config_module.config());

            if let Some(write_buf) = write_buf {
                writeln!(write_buf, "{}", config_msg)?;
                writeln!(write_buf, "{}", n_plus_message)?;
            }

            tracing::info!("{}", config_msg);
            tracing::info!("{}", n_plus_message);
            // Check the endpoints' schema
            let _ = config_module
                .extensions()
                .endpoint_set
                .clone()
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
