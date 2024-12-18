use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;

use super::helpers::TRACKER;
use super::validate_rc::validate_rc_config_files;
use super::{check, gen, init, start};
use crate::cli::command::{Cli, Command};
use crate::cli::{self, update_checker};
use crate::core::blueprint::Blueprint;
use crate::core::config::reader::ConfigReader;
use crate::core::runtime::TargetRuntime;

pub async fn run() -> Result<()> {
    if let Ok(path) = dotenv() {
        tracing::info!("Env file: {:?} loaded", path);
    }
    let cli = Cli::parse();
    tokio::task::spawn(update_checker::check_for_update());
    // Initialize ping event every 60 seconds
    let _ = TRACKER
        .init_ping(tokio::time::Duration::from_secs(60))
        .await;

    // Dispatch the command as an event
    let _ = TRACKER
        .dispatch(tailcall_tracker::EventKind::Command(
            cli.command.to_string(),
        ))
        .await;

    run_command(cli).await
}

fn get_runtime_and_config_reader(verify_ssl: bool) -> (TargetRuntime, ConfigReader) {
    let mut blueprint = Blueprint::default();
    blueprint.upstream.verify_ssl = verify_ssl;
    let runtime = cli::runtime::init(&blueprint);
    (runtime.clone(), ConfigReader::init(runtime))
}

async fn run_command(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Start { file_paths, verify_ssl } => {
            let (runtime, config_reader) = get_runtime_and_config_reader(verify_ssl);
            validate_rc_config_files(runtime, &file_paths).await;
            start::start_command(file_paths, &config_reader).await?;
        }
        Command::Check { file_paths, n_plus_one_queries, schema, verify_ssl } => {
            let (runtime, config_reader) = get_runtime_and_config_reader(verify_ssl);
            validate_rc_config_files(runtime.clone(), &file_paths).await;
            check::check_command(
                check::CheckParams { file_paths, n_plus_one_queries, schema, runtime },
                &config_reader,
            )
            .await?;
        }
        Command::Init { folder_path } => {
            let (runtime, _) = get_runtime_and_config_reader(true);
            init::init_command(runtime, &folder_path).await?;
        }
        Command::Gen { file_path } => {
            let (runtime, _) = get_runtime_and_config_reader(true);
            gen::gen_command(&file_path, runtime).await?;
        }
    }
    Ok(())
}
