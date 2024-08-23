use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::time::Instant;

use super::helpers::log_endpoint_set;
use crate::cli::fmt::Fmt;
use crate::cli::server::http_server::RUNTIME;
use crate::cli::server::Server;
use crate::core::config::reader::ConfigReader;
use crate::core::config::ConfigModule;

pub(super) async fn start_command(
    file_paths: Vec<String>,
    watch: bool,
    config_reader: ConfigReader,
) -> Result<()> {
    if watch {
        start_watch_server(&file_paths, config_reader).await?;
    } else {
        let config_module = config_reader
            .read_all(&file_paths)
            .await
            .context("Failed to read config files")?;
        log_endpoint_set(&config_module.extensions().endpoint_set);
        Fmt::log_n_plus_one(false, config_module.config());
        let server = Server::new(config_module);
        server
            .fork_start(false)
            .await
            .context("Failed to start server")?;
    }
    Ok(())
}

/// Starts the server in watch mode
async fn start_watch_server(file_paths: &[String], config_reader: ConfigReader) -> Result<()> {
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
    let mut watcher =
        RecommendedWatcher::new(watch_tx, Config::default()).context("Failed to create watcher")?;

    for path in file_paths {
        if let Err(err) = watcher.watch(path.as_ref(), RecursiveMode::NonRecursive) {
            tracing::error!("Failed to watch path {:?}: {}", path, err);
        }
    }

    // Debounce delay to prevent multiple server restarts on a single file change
    // https://users.rust-lang.org/t/problem-with-notify-crate-v6-1/99877
    let debounce_duration = Duration::from_secs(1);
    // ensures the first server start is not blocked
    let mut last_event_time = Instant::now() - (debounce_duration * 4);
    let arc_config_reader = Arc::new(config_reader);
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

                            tracing::info!("Restarting server");
                            if let Some(runtime) = RUNTIME.lock().unwrap().take() {
                                runtime.shutdown_background();
                            }

                            handle_watch_server(
                                file_paths,
                                arc_config_reader.clone(),
                                &mut watcher,
                            )
                            .await
                            .context("Failed to handle watch server")?;
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
async fn handle_watch_server(
    file_paths: &[String],
    config_reader: Arc<ConfigReader>,
    watcher: &mut RecommendedWatcher,
) -> Result<()> {
    if file_paths.len() == 1 {
        match config_reader.read_all(file_paths).await {
            Ok(config_module) => {
                watch_linked_files(file_paths[0].as_str(), config_module.clone(), watcher).await;
                log_endpoint_set(&config_module.extensions().endpoint_set);
                Fmt::log_n_plus_one(false, config_module.config());

                let server = Server::new(config_module);
                if let Err(err) = server.fork_start(true).await {
                    tracing::error!("Failed to start server: {}", err);
                }
            }
            Err(err) => {
                tracing::error!("{}", err);
            }
        }
    } else {
        // ensure to watch for correct linked files wrt the config file
        for file in file_paths {
            match config_reader.read(file.as_str()).await {
                Ok(config_module) => {
                    watch_linked_files(file, config_module, watcher).await;
                }
                Err(err) => {
                    tracing::error!("Failed to read config files: {}", err);
                }
            }
        }
        match config_reader.read_all(file_paths).await {
            Ok(config_module) => {
                log_endpoint_set(&config_module.extensions().endpoint_set);
                Fmt::log_n_plus_one(false, config_module.config());

                let server = Server::new(config_module);
                if let Err(err) = server.fork_start(true).await {
                    tracing::error!("Failed to start server: {}", err);
                }
            }
            Err(err) => {
                tracing::error!("Failed to read config files: {}", err);
            }
        }
    }
    Ok(())
}

async fn watch_linked_files(
    file_path: &str,
    config_module: ConfigModule,
    watcher: &mut RecommendedWatcher,
) {
    let links = config_module
        .links
        .iter()
        .map(|link| link.src.clone())
        .collect::<Vec<_>>();
    for link in links {
        let mut link_path = link.clone();
        if let Some(pos) = file_path.rfind('/') {
            let root_dir = Path::new(&file_path[..pos]);
            link_path = ConfigReader::resolve_path(&link, Some(root_dir));
        } else if let Ok(current_dir) = std::env::current_dir() {
            link_path = ConfigReader::resolve_path(&link, Some(current_dir.as_path()));
        }
        if let Err(err) = watcher.watch(link_path.as_ref(), RecursiveMode::NonRecursive) {
            tracing::error!("Failed to watch path {:?}: {}", link, err);
        }
    }
}
