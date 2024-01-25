use std::sync::Arc;

use anyhow::Result;
use corex::blueprint::{Blueprint, Http};
use corex::config::Config;
use tokio::sync::oneshot::{self};

use super::http_1::start_http_1;
use super::http_2::start_http_2;
use super::server_config::ServerConfig;
use crate::cli::CLIError;

pub struct Server {
  config: Config,
  server_up_sender: Option<oneshot::Sender<()>>,
}

impl Server {
  pub fn new(config: Config) -> Self {
    Self { config, server_up_sender: None }
  }

  pub fn server_up_receiver(&mut self) -> oneshot::Receiver<()> {
    let (tx, rx) = oneshot::channel();

    self.server_up_sender = Some(tx);

    rx
  }

  /// Starts the server in the current Runtime
  pub async fn start(self) -> Result<()> {
    let blueprint = Blueprint::try_from(&self.config).map_err(CLIError::from)?;
    let server_config = Arc::new(ServerConfig::new(blueprint.clone()));

    match blueprint.server.http.clone() {
      Http::HTTP2 { cert, key } => start_http_2(server_config, cert, key, self.server_up_sender).await,
      Http::HTTP1 => start_http_1(server_config, self.server_up_sender).await,
    }
  }

  /// Starts the server in its own multithreaded Runtime
  pub async fn fork_start(self) -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
      .worker_threads(self.config.server.get_workers())
      .enable_all()
      .build()?;

    let result = runtime.spawn(async { self.start().await }).await?;
    runtime.shutdown_background();

    result
  }
}
