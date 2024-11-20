use std::path::Path;

use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;

use super::helpers::{TAILCALL_RC, TAILCALL_RC_SCHEMA, TRACKER};
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
    update_checker::check_for_update().await;
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
        Command::Check { file_paths, n_plus_one_queries, schema, format, verify_ssl } => {
            let (runtime, config_reader) = get_runtime_and_config_reader(verify_ssl);
            validate_rc_config_files(runtime.clone(), &file_paths).await;
            check::check_command(
                check::CheckParams { file_paths, n_plus_one_queries, schema, format, runtime },
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

async fn validate_rc_config_files(runtime: TargetRuntime, file_paths: &[String]) {
    // base config files.
    let tailcallrc = include_str!("../../../generated/.tailcallrc.graphql");
    let tailcallrc_json = include_str!("../../../generated/.tailcallrc.schema.json");

    for path in file_paths {
        let tailcall_rc_path = Path::new(path).join(TAILCALL_RC).display().to_string();
        let tailcall_rc_schema_path = Path::new(path)
            .join(TAILCALL_RC_SCHEMA)
            .display()
            .to_string();

        // check if rc files already exist or not.
        if std::fs::metadata(tailcall_rc_path.clone()).is_ok()
            || std::fs::metadata(tailcall_rc_schema_path.clone()).is_ok()
        {
            let mut outdated_files = Vec::with_capacity(2);
            if let Ok(content) = runtime.file.read(&tailcall_rc_path).await {
                if content != tailcallrc {
                    outdated_files.push(".tailcallrc.graphql");
                }
            } else {
                // If unable to read the file, consider it outdated
                outdated_files.push(".tailcallrc.graphql");
            }
            // Check .tailcallrc.schema.json
            if let Ok(content) = runtime.file.read(&tailcall_rc_schema_path).await {
                if content != tailcallrc_json {
                    outdated_files.push(".tailcallrc.schema.json");
                }
            } else {
                // If unable to read the file, consider it outdated
                outdated_files.push(".tailcallrc.schema.json");
            }

            if !outdated_files.is_empty() {
                let message = if outdated_files.len() == 2 {
                    format!(
                        "[{}, {}] is outdated, reinitialize using tailcall init.",
                        outdated_files[0], outdated_files[1]
                    )
                } else {
                    format!(
                        "[{}] is outdated, reinitialize using tailcall init.",
                        outdated_files[0]
                    )
                };
                tracing::warn!(message);
                return;
            }
        }
    }
}
