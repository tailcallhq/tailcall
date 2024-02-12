use std::path::Path;
use std::{env, fs};

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use inquire::Confirm;
use stripmargin::StripMargin;

use super::command::{Cli, Command};
use super::update_checker;
use crate::blueprint::{OperationQuery, Upstream};
use crate::builder::TailcallBuilder;
use crate::cli::server::Server;
use crate::cli::{self};

const FILE_NAME: &str = ".tailcallrc.graphql";
const YML_FILE_NAME: &str = ".graphqlrc.yml";

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    logger_init();
    update_checker::check_for_update().await;
    let runtime = cli::runtime::init(&Upstream::default(), None);
    let tailcall_builder = TailcallBuilder::init(runtime.clone());
    match cli.command {
        Command::Start { file_paths } => {
            let tailcall_executor = tailcall_builder.with_config_paths(&file_paths).await?;

            log::info!(
                "N + 1: {}",
                tailcall_executor
                    .config_module
                    .n_plus_one()
                    .len()
                    .to_string()
            );
            let server = Server::new(tailcall_executor);
            server.fork_start().await?;
            Ok(())
        }
        Command::Check { file_paths, n_plus_one_queries, schema, operations } => {
            let ops: Vec<OperationQuery> =
                futures_util::future::join_all(operations.iter().map(|op| async {
                    runtime
                        .file
                        .read(op)
                        .await
                        .map(|query| OperationQuery::new(query, op.clone()))
                }))
                .await
                .into_iter()
                .collect::<Result<Vec<_>>>()?;

            let tailcall_executor = tailcall_builder
                .with_config_paths(&file_paths)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let result = tailcall_executor
                .validate(n_plus_one_queries, schema, ops)
                .await?;
            println!("{}", result);
            Ok(())
        }
        Command::Init { folder_path } => init(&folder_path).await,
        Command::Compose { file_paths, format } => {
            let executor = tailcall_builder.with_config_paths(&file_paths).await?;
            let encoded = format.encode(&executor.config_module)?;
            println!("{encoded}");
            Ok(())
        }
    }
}

pub async fn init(folder_path: &str) -> Result<()> {
    let folder_exists = fs::metadata(folder_path).is_ok();

    if !folder_exists {
        let confirm = Confirm::new(&format!(
            "Do you want to create the folder {}?",
            folder_path
        ))
        .with_default(false)
        .prompt()?;

        if confirm {
            fs::create_dir_all(folder_path)?;
        } else {
            return Ok(());
        };
    }

    let tailcallrc = include_str!("../../generated/.tailcallrc.graphql");

    let file_path = Path::new(folder_path).join(FILE_NAME);
    let yml_file_path = Path::new(folder_path).join(YML_FILE_NAME);

    let tailcall_exists = fs::metadata(&file_path).is_ok();

    if tailcall_exists {
        // confirm overwrite
        let confirm = Confirm::new(&format!("Do you want to overwrite the file {}?", FILE_NAME))
            .with_default(false)
            .prompt()?;

        if confirm {
            fs::write(&file_path, tailcallrc.as_bytes())?;
        }
    } else {
        fs::write(&file_path, tailcallrc.as_bytes())?;
    }

    let yml_exists = fs::metadata(&yml_file_path).is_ok();

    if !yml_exists {
        fs::write(&yml_file_path, "")?;

        let graphqlrc = r"|schema:
         |- './.tailcallrc.graphql'
    "
        .strip_margin();

        fs::write(&yml_file_path, graphqlrc)?;
    }

    let graphqlrc = fs::read_to_string(&yml_file_path)?;

    let file_path = file_path.to_str().unwrap();

    let mut yaml: serde_yaml::Value = serde_yaml::from_str(&graphqlrc)?;

    if let Some(mapping) = yaml.as_mapping_mut() {
        let schema = mapping
            .entry("schema".into())
            .or_insert(serde_yaml::Value::Sequence(Default::default()));
        if let Some(schema) = schema.as_sequence_mut() {
            if !schema
                .iter()
                .any(|v| v == &serde_yaml::Value::from("./.tailcallrc.graphql"))
            {
                let confirm =
                    Confirm::new(&format!("Do you want to add {} to the schema?", file_path))
                        .with_default(false)
                        .prompt()?;

                if confirm {
                    schema.push(serde_yaml::Value::from("./.tailcallrc.graphql"));
                    let updated = serde_yaml::to_string(&yaml)?;
                    fs::write(yml_file_path, updated)?;
                }
            }
        }
    }

    Ok(())
}

// initialize logger
fn logger_init() {
    // set the log level
    const LONG_ENV_FILTER_VAR_NAME: &str = "TAILCALL_LOG_LEVEL";
    const SHORT_ENV_FILTER_VAR_NAME: &str = "TC_LOG_LEVEL";

    // Select which env variable to use for the log level filter. This is because filter_or doesn't allow picking between multiple env_var for the filter value
    let filter_env_name = env::var(LONG_ENV_FILTER_VAR_NAME)
        .map(|_| LONG_ENV_FILTER_VAR_NAME)
        .unwrap_or_else(|_| SHORT_ENV_FILTER_VAR_NAME);

    // use the log level from the env if there is one, otherwise use the default.
    let env = Env::new().filter_or(filter_env_name, "info");

    env_logger::Builder::from_env(env).init();
}
