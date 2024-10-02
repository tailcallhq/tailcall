use std::io::BufWriter;

use anyhow::Result;

use super::helpers::{display_schema, log_endpoint_set};
use crate::cli::fmt::Fmt;
use crate::core::blueprint::Blueprint;
use crate::core::config::reader::ConfigReader;
use crate::core::config::Source;
use crate::core::runtime::TargetRuntime;
use crate::core::Errata;

pub struct CheckParams {
    pub file_paths: Vec<String>,
    pub n_plus_one_queries: bool,
    pub schema: bool,
    pub format: Option<Source>,
    pub runtime: TargetRuntime,
}

pub async fn check_command(params: CheckParams, config_reader: &ConfigReader) -> Result<()> {
    let CheckParams { file_paths, n_plus_one_queries, schema, format, runtime } = params;
    let mut fmt_std = Fmt::new(std::io::stdout().lock());
    let mut fmt_vec = Fmt::new(BufWriter::new(Vec::new()));

    let config_module = (config_reader.read_all(&file_paths)).await?;
    log_endpoint_set(&config_module.extensions().endpoint_set);
    if let Some(format) = format {
        fmt_std.append(&format.encode(&config_module)?)?;
        fmt_std.display_and_drop()?;
    }
    let blueprint = Blueprint::try_from(&config_module).map_err(Errata::from);

    match blueprint {
        Ok(blueprint) => {
            tracing::info!("Config {} ... ok", file_paths.join(", "));
            fmt_vec.log_n_plus_one(n_plus_one_queries, config_module.config())?;
            let out = fmt_vec.display()?;
            let output = out.into_inner()?;
            tracing::info!("{}", String::from_utf8(output)?);

            // Check the endpoints' schema
            let _ = config_module
                .extensions()
                .endpoint_set
                .clone()
                .into_checked(&blueprint, runtime)
                .await?;
            if schema {
                display_schema(&blueprint)?;
            }

            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
