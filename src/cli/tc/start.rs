use std::io::BufWriter;

use anyhow::Result;

use super::helpers::log_endpoint_set;
use crate::cli::fmt::Fmt;
use crate::cli::server::Server;
use crate::core::config::reader::ConfigReader;

pub(super) async fn start_command(
    file_paths: Vec<String>,
    config_reader: &ConfigReader,
) -> Result<()> {
    let mut fmt = Fmt::new(BufWriter::new(vec![]));

    let config_module = config_reader.read_all(&file_paths).await?;
    log_endpoint_set(&config_module.extensions().endpoint_set);
    fmt.log_n_plus_one(false, config_module.config())?;
    let vec = fmt.display()?;
    let output = vec.into_inner()?;
    tracing::info!("{}", String::from_utf8(output)?);

    let server = Server::new(config_module);
    server.fork_start().await?;
    Ok(())
}
