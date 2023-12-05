use std::sync::Arc;

use anyhow::Result;
use tokio::sync::oneshot::{self};

use super::http_1::start_http_1;
use super::http_2::start_http_2;
use super::server_config::{ServerConfig};
use super::ServerMessage;
use crate::blueprint::{Blueprint, Http};
use crate::cli::CLIError;
use crate::config::Config;

pub async fn start_server(
  config: Config,
  server_up_sender: oneshot::Sender<ServerMessage>,
  shutdown_receiver: oneshot::Receiver<ServerMessage>,
) -> Result<()> {
  let blueprint = Blueprint::try_from(&config).map_err(CLIError::from)?;
  let server_config = Arc::new(ServerConfig::new(blueprint.clone()));

  match blueprint.server.http.clone() {
    Http::HTTP2 { cert, key } => start_http_2(server_config, cert, key, server_up_sender, shutdown_receiver).await,
    Http::HTTP1 => start_http_1(server_config, server_up_sender, shutdown_receiver).await,
  }
}
