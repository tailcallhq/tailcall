use std::sync::Arc;

use anyhow::{Context, Result};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::broadcast;
use tokio::task;

use super::helpers::log_endpoint_set;
use crate::cli::fmt::Fmt;
use crate::cli::server::Server;
use crate::core::config::reader::ConfigReader;

pub(super) async fn start_command(
    file_paths: Vec<String>,
    watch: bool,
    config_reader: Arc<ConfigReader>,
) -> Result<()> {
    if watch {
        start_watch_server(file_paths, config_reader).await?;
    } else {
        let config_module = config_reader
            .read_all(&file_paths)
            .await
            .context("Failed to read config files")?;
        log_endpoint_set(&config_module.extensions().endpoint_set);
        Fmt::log_n_plus_one(false, config_module.config());
        let server = Server::new(config_module);
        server
            .fork_start(None)
            .await
            .context("Failed to start server")?;
    }
    Ok(())
}

async fn start_watch_server(
    file_paths: Vec<String>,
    config_reader: Arc<ConfigReader>,
) -> Result<()> {
    let (tx, mut rx) = broadcast::channel(16);
    let file_paths_clone = file_paths.clone();

    let watch_handler = task::spawn(async move {
        let (watch_tx, watch_rx) = std::sync::mpsc::channel();

        let mut watcher = match RecommendedWatcher::new(watch_tx, Config::default()) {
            Ok(watcher) => watcher,
            Err(err) => {
                tracing::error!("Failed to create watcher: {}", err);
                return;
            }
        };

        for path in &file_paths_clone {
            if let Err(err) = watcher.watch(path.as_ref(), RecursiveMode::Recursive) {
                tracing::error!("Failed to watch path {:?}: {}", path, err);
            }
        }

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            match watch_rx.recv() {
                Ok(event) => {
                    if let Ok(event) = event {
                        if let notify::EventKind::Modify(notify::event::ModifyKind::Data(
                            notify::event::DataChange::Content,
                        )) = event.kind
                        {
                            tracing::info!("File change detected");
                            if let Err(err) = tx.send(()) {
                                tracing::error!("Failed to send the signal: {}", err);
                            }
                        }
                    }
                }
                Err(e) => tracing::error!("Watch error: {:?}", e),
            }
        }
    });

    let server_handler = task::spawn({
        let config_reader = Arc::clone(&config_reader);
        let file_paths = file_paths.clone();
        async move {
            let mut rec = Some(&mut rx);
            let mut config_error = false;
            loop {
                match config_reader.read_all(&file_paths).await {
                    Ok(config_module) => {
                        log_endpoint_set(&config_module.extensions().endpoint_set);
                        Fmt::log_n_plus_one(false, config_module.config());
                        let server = Server::new(config_module.clone());
                        if let Err(err) = server.fork_start(rec.as_deref_mut()).await {
                            tracing::error!("Failed to start server: {}", err);
                        }
                        config_error = false;
                        tracing::info!("Restarting server");
                    }
                    Err(err) => {
                        if !config_error {
                            tracing::error!("Failed to read config files: {}", err);
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            config_error = true;
                        }
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    });

    if let Err(err) = watch_handler.await {
        tracing::debug!("Error in watch handler: {}", err);
    }
    if let Err(err) = server_handler.await {
        tracing::debug!("Error in server handler: {}", err);
    }
    Ok(())
}
