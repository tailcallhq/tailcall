use std::sync::Arc;

use anyhow::Result;
use tokio::sync::oneshot::{self};

use super::http_1::start_http_1;
use super::http_2::start_http_2;
use super::server_config::ServerConfig;
use crate::blueprint::Http;
use crate::runtime::TargetRuntime;
use crate::TailcallBuilder;

pub struct Server {
    tailcall_builder: TailcallBuilder,
    server_up_sender: Option<oneshot::Sender<()>>,
}

impl Server {
    pub fn new(tailcall_builder: TailcallBuilder) -> Self {
        Self { tailcall_builder, server_up_sender: None }
    }

    pub fn server_up_receiver(&mut self) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();

        self.server_up_sender = Some(tx);

        rx
    }

    /// Starts the server in the current Runtime
    pub async fn start(self, runtime: TargetRuntime) -> Result<()> {
        let server_config = Arc::new(ServerConfig::new(self.tailcall_builder, runtime).await?);

        match server_config
            .tailcall_executor
            .app_ctx
            .blueprint
            .server
            .http
            .clone()
        {
            Http::HTTP2 { cert, key } => {
                start_http_2(server_config, cert, key, self.server_up_sender).await
            }
            Http::HTTP1 => start_http_1(server_config, self.server_up_sender).await,
        }
    }

    /// Starts the server in its own multithreaded Runtime
    pub async fn fork_start(self, target_runtime: TargetRuntime) -> Result<()> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(
                self.tailcall_builder
                    .get_blueprint(&target_runtime)
                    .await?
                    .server
                    .worker,
            )
            .enable_all()
            .build()?;

        let result = runtime
            .spawn(async { self.start(target_runtime).await })
            .await?;
        runtime.shutdown_background();

        result
    }
}
