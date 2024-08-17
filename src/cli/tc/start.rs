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

/// Starts the server in watch mode
async fn start_watch_server(
    file_paths: Vec<String>,
    config_reader: Arc<ConfigReader>,
) -> Result<()> {
    let (tx, rx) = broadcast::channel(16);

    let watch_handler = task::spawn({
        let config_reader = Arc::clone(&config_reader);
        async move {
            if let Err(err) = watch_files(&file_paths, tx, rx, config_reader).await {
                tracing::error!("Watch handler encountered an error: {}", err);
            }
        }
    });

    if let Err(err) = watch_handler.await {
        tracing::error!("Error in watch handler: {}", err);
    }

    Ok(())
}

/// Watches the file paths for changes
async fn watch_files(
    file_paths: &[String],
    tx: broadcast::Sender<()>,
    mut rx: broadcast::Receiver<()>,
    config_reader: Arc<ConfigReader>,
) -> Result<()> {
    let (watch_tx, watch_rx) = std::sync::mpsc::channel();
    // fake event to trigger the first server start
    watch_tx
        .clone()
        .send(Ok(notify::event::Event::new(
            notify::event::EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
        )))
        .unwrap();
    let mut watcher = match RecommendedWatcher::new(watch_tx, Config::default()) {
        Ok(watcher) => watcher,
        Err(err) => {
            tracing::error!("Failed to create watcher: {}", err);
            return Ok(());
        }
    };

    for path in file_paths {
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
                        if let Err(err) = tx.send(()) {
                            tracing::error!("Failed to send the signal: {}", err);
                        }
                        let _ = handle_server(&mut rx, file_paths, config_reader.clone()).await;
                    }
                }
            }
            Err(e) => tracing::error!("Watch error: {:?}", e),
        }
    }
}

/// Handles the server (in watch mode)
/// Prevents server crashes if config reader fails
async fn handle_server(
    rx: &mut broadcast::Receiver<()>,
    file_paths: &[String],
    config_reader: Arc<ConfigReader>,
) -> Result<()> {
    match config_reader.read_all(file_paths).await {
        Ok(config_module) => {
            log_endpoint_set(&config_module.extensions().endpoint_set);
            Fmt::log_n_plus_one(false, config_module.config());

            let server = Server::new(config_module.clone());
            if let Err(err) = server.fork_start(Some(rx)).await {
                tracing::error!("Failed to start server: {}", err);
            }
        }
        Err(err) => {
            tracing::error!("Failed to read config files: {}", err);
        }
    }
    Ok(())
}
