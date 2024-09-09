use anyhow::Result;
use clap::Parser;
use convert_case::{Case, Casing};
use dotenvy::dotenv;

use super::helpers::TRACKER;
use super::{check, gen, init, start};
use crate::cli::command::{Cli, Command};
use crate::cli::{self, update_checker};
use crate::core::blueprint::Blueprint;
use crate::core::config::reader::ConfigReader;
use crate::core::runtime::TargetRuntime;
use std::fs;
use std::path::Path;



pub async fn run() -> Result<()> {
    if let Ok(path) = dotenv() {
        tracing::info!("Env file: {:?} loaded", path);
    }
    let cli = Cli::parse();
    update_checker::check_for_update().await;
    let runtime = cli::runtime::init(&Blueprint::default());
    let config_reader = ConfigReader::init(runtime.clone());

    // Initialize ping event every 60 seconds
    let _ = TRACKER
        .init_ping(tokio::time::Duration::from_secs(60))
        .await;

    // Dispatch the command as an event
    let _ = TRACKER
        .dispatch(cli.command.to_string().to_case(Case::Snake).as_str())
        .await;

    run_command(cli, config_reader, runtime).await
}

fn get_absolute_paths(file_paths: Vec<String>) -> Vec<String> {
    file_paths
        .into_iter()
        .map(|path| fs::canonicalize(Path::new(&path))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path)) // Fallback to original path if conversion fails
        .collect()
}

async fn run_command(cli: Cli, config_reader: ConfigReader, runtime: TargetRuntime) -> Result<()> {
    match cli.command {
        Command::Start { file_paths } => {
            let absolute_paths = get_absolute_paths(file_paths);
            start::start_command(absolute_paths, &config_reader).await?;
        }
        Command::Check { file_paths, n_plus_one_queries, schema, format } => {
            let file_paths = get_absolute_paths(file_paths);
            check::check_command(
                check::CheckParams { file_paths, n_plus_one_queries, schema, format, runtime },
                &config_reader,
            )
            .await?;
        }
        Command::Init { folder_path } => {
            init::init_command(runtime, &folder_path).await?;
        }
        Command::Gen { file_path } => {
            gen::gen_command(&file_path, runtime).await?;
        }
    }
    Ok(())
}
