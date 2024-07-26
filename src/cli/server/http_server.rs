use std::cell::Cell;
use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::oneshot::{self};
use tracing::subscriber::DefaultGuard;

use super::http_1::start_http_1;
use super::http_2::start_http_2;
use super::server_config::ServerConfig;
use crate::cli::telemetry::init_opentelemetry;
use crate::cli::CLIError;
use crate::core::blueprint::{Blueprint, Http};
use crate::core::config::ConfigModule;
thread_local! {
    static TRACING_GUARD: Cell<Option<DefaultGuard>> = const { Cell::new(None) };
}

pub struct Server {
    config_module: ConfigModule,
    server_up_sender: Option<oneshot::Sender<()>>,
}

impl Server {
    pub fn new(config_module: ConfigModule) -> Self {
        Self { config_module, server_up_sender: None }
    }

    pub fn server_up_receiver(&mut self) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();

        self.server_up_sender = Some(tx);

        rx
    }

    /// Starts the server in the current Runtime
    pub async fn start(self) -> Result<()> {
        let blueprint = Blueprint::try_from(&self.config_module).map_err(CLIError::from)?;
        let endpoints = self.config_module.extensions().endpoint_set.clone();
        let server_config = Arc::new(ServerConfig::new(blueprint.clone(), endpoints).await?);

        init_opentelemetry(blueprint.telemetry.clone(), &server_config.app_ctx.runtime)?;

        match blueprint.server.http.clone() {
            Http::HTTP2 { cert, key } => {
                start_http_2(server_config, cert, key, self.server_up_sender).await
            }
            Http::HTTP1 => start_http_1(server_config, self.server_up_sender).await,
        }
    }

    /// Starts the server in its own multithreaded Runtime
    pub async fn fork_start(self) -> Result<()> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.config_module.deref().server.get_workers())
            .on_thread_start(|| {
                // initialize default tracing setup for the cli execution for every thread that
                // is spawned based on https://github.com/tokio-rs/tracing/issues/593#issuecomment-589857097
                // and required due to the fact that later for tracing the global subscriber
                // will be set by `src/cli/opentelemetry.rs` and until that we need
                // to use the default tracing configuration for cli output. And
                // since `set_default` works only for current thread incorporate this
                // with tokio runtime
                let guard = tracing::subscriber::set_default(
                    crate::core::tracing::default_tracing_tailcall(),
                );

                TRACING_GUARD.set(Some(guard));
            })
            .on_thread_stop(|| {
                TRACING_GUARD.take();
            })
            .enable_all()
            .build()?;

        let result = runtime.spawn(self.start()).await?;
        runtime.shutdown_background();

        result
    }
}
