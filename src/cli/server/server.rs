use std::sync::Arc;

use anyhow::Result;
use tokio::sync::oneshot::{self};

use super::http_1::start_http_1;
use super::http_2::start_http_2;
use super::server_config::ServerConfig;
use crate::blueprint::{Blueprint, Http};
use crate::cli::CLIError;
use crate::cli::opentelemetry::init_opentelemetry;
use crate::config::Config;

pub struct Server {
  blueprint: Blueprint,
  server_up_sender: Option<oneshot::Sender<()>>,
}

impl Server {
  pub fn try_new(config: Config) -> Result<Self> {
    let blueprint = Blueprint::try_from(&config).map_err(CLIError::from)?;

    init_opentelemetry(blueprint.opentelemetry.clone())?;

    Ok(Self { blueprint, server_up_sender: None })
  }

  pub fn server_up_receiver(&mut self) -> oneshot::Receiver<()> {
    let (tx, rx) = oneshot::channel();

    self.server_up_sender = Some(tx);

    rx
  }

  pub async fn start(self) -> Result<()> {
    let server_config = Arc::new(ServerConfig::new(self.blueprint.clone()));

    match self.blueprint.server.http.clone() {
      Http::HTTP2 { cert, key } => start_http_2(server_config, cert, key, self.server_up_sender).await,
      Http::HTTP1 => start_http_1(server_config, self.server_up_sender).await,
    }
  }
}
