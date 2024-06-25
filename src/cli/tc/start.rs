use super::helpers::log_endpoint_set;
use super::cli::fmt::Fmt;
use super::cli::server::Server;
use crate::core::config::reader::ConfigReader;
use anyhow::Result;

pub(super) async fn start_command(file_paths: Vec<String>, config_reader: &ConfigReader) -> Result<()> {
    let config_module = config_reader.read_all(&file_paths).await?;
    log_endpoint_set(&config_module.extensions.endpoint_set);
    Fmt::log_n_plus_one(false, &config_module.config);
    let server = Server::new(config_module);
    server.fork_start().await?;
    Ok(())
}

