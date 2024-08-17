use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use notify::{Config, EventKind, FsEventWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::broadcast;
use tokio::time::Instant;

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

    watch_files(&file_paths, tx, rx, config_reader).await?;

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
        if let Err(err) = watcher.watch(path.as_ref(), RecursiveMode::NonRecursive) {
            tracing::error!("Failed to watch path {:?}: {}", path, err);
        }
    }

    // Debounce delay to prevent multiple server restarts on a single file change
    // https://users.rust-lang.org/t/problem-with-notify-crate-v6-1/99877
    let debounce_duration = Duration::from_secs(1);
    let mut last_event_time = Instant::now() - debounce_duration;
    loop {
        match watch_rx.recv() {
            Ok(event) => {
                if let Ok(event) = event {
                    if let EventKind::Modify(notify::event::ModifyKind::Data(
                        notify::event::DataChange::Content,
                    )) = event.kind
                    {
                        let now = Instant::now();
                        if now.duration_since(last_event_time) >= debounce_duration {
                            last_event_time = now;

                            if let Err(err) = tx.send(()) {
                                tracing::error!("Failed to send the signal: {}", err);
                            }

                            if let Err(e) = handle_server(
                                &mut rx,
                                file_paths,
                                config_reader.clone(),
                                &mut watcher,
                            )
                            .await
                            {
                                tracing::error!("Failed to handle server: {}", e);
                            }
                        }
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
    watcher: &mut FsEventWatcher,
) -> Result<()> {
    match config_reader.read_all(file_paths).await {
        Ok(config_module) => {
            let links = config_module
                .clone()
                .links
                .iter()
                .map(|link| link.src.clone())
                .collect::<Vec<_>>();
            for link in links {
                if let Err(err) = watcher.watch(link.as_ref(), RecursiveMode::NonRecursive) {
                    tracing::error!("Failed to watch path {:?}: {}", link, err);
                }
            }
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
