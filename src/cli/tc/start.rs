use std::path::Path;
use std::sync::mpsc::{channel, Receiver};

use anyhow::Result;
use notify::{recommended_watcher, Event, RecursiveMode, Watcher};

use super::helpers::log_endpoint_set;
use crate::cli::fmt::Fmt;
use crate::cli::server::Server;
use crate::core::config::reader::ConfigReader;

async fn run_server(file_paths: Vec<String>, config_reader: &ConfigReader) -> Result<()> {
    let config_module = config_reader.read_all(&file_paths).await?;
    log_endpoint_set(&config_module.extensions().endpoint_set);
    Fmt::log_n_plus_one(false, config_module.config());
    let server = Server::new(config_module);
    server.fork_start().await?;
    Ok(())
}

pub(super) async fn start_command(
    file_paths: Vec<String>,
    config_reader: &ConfigReader,
    watch: bool,
) -> Result<()> {
    if watch {
        let (tx, rx): (std::sync::mpsc::Sender<Event>, Receiver<Event>) = channel();

        let tx_clone = tx.clone();

        let mut watcher = recommended_watcher(move |res| match res {
            Ok(event) => {
                tx_clone.send(event).expect("Failed to send event");
            }
            Err(e) => tracing::error!("Watch error: {:?}", e),
        })?;

        for path in &file_paths {
            watcher.watch(Path::new(path), RecursiveMode::Recursive)?;
        }

        run_server(file_paths.clone(), config_reader).await?;

        loop {
            match rx.recv() {
                Ok(event) => {
                    tracing::info!("File change detected: {:?}", event);
                    run_server(file_paths.clone(), config_reader).await?;
                }
                Err(e) => tracing::error!("Watch error: {:?}", e),
            }
        }
    } else {
        run_server(file_paths, config_reader).await?;
    }

    Ok(())
}
