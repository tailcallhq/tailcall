use std::path::Path;
use std::sync::Arc;
use std::{env, fs};

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use inquire::Confirm;
use stripmargin::StripMargin;

use super::command::{Cli, Command};
use crate::blueprint::{validate_operations, Blueprint, OperationQuery};
use crate::cli::fmt::Fmt;
use crate::cli::server::Server;
use crate::cli::{init_file, init_http, init_proto_resolver, CLIError};
use crate::config::reader::ConfigReader;
use crate::config::{Config, Upstream};
use crate::{print_schema, FileIO};

const FILE_NAME: &str = ".tailcallrc.graphql";
const YML_FILE_NAME: &str = ".graphqlrc.yml";

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    logger_init();
    let file_io: Arc<dyn FileIO> = init_file();
    let default_http_io = init_http(&Upstream::default(), None);
    let config_reader = ConfigReader::init(file_io.clone(), default_http_io.clone());
    match cli.command {
        Command::Start { file_paths } => {
            let config_set = config_reader.read_all(&file_paths).await?;
            let config_set = config_set
                .resolve_extensions(file_io.clone(), default_http_io, init_proto_resolver())
                .await;
            log::info!("N + 1: {}", config_set.n_plus_one().len().to_string());
            let server = Server::new(config_set);
            server.fork_start().await?;
            Ok(())
        }
        Command::Check { file_paths, n_plus_one_queries, schema, operations } => {
            let config_set = (config_reader.read_all(&file_paths)).await?;
            let config_set = config_set
                .resolve_extensions(file_io.clone(), default_http_io, init_proto_resolver())
                .await;
            let blueprint = Blueprint::try_from(&config_set).map_err(CLIError::from);

            match blueprint {
                Ok(blueprint) => {
                    log::info!("{}", "Config successfully validated".to_string());
                    display_config(&config_set, n_plus_one_queries);
                    if schema {
                        display_schema(&blueprint);
                    }

                    let ops: Vec<OperationQuery> =
                        futures_util::future::join_all(operations.iter().map(|op| async {
                            file_io
                                .read(op)
                                .await
                                .map(|query| OperationQuery::new(query, op.clone()))
                        }))
                        .await
                        .into_iter()
                        .collect::<Result<Vec<_>>>()?;

                    validate_operations(&blueprint, ops)
                        .await
                        .to_result()
                        .map_err(|e| {
                            CLIError::from(e)
                                .message("Invalid Operation".to_string())
                                .into()
                        })
                }
                Err(e) => Err(e.into()),
            }
        }
        Command::Init { folder_path } => init(&folder_path).await,
        Command::Compose { file_paths, format } => {
            let config = (config_reader.read_all(&file_paths).await)?;
            Fmt::display(format.encode(&config)?);
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

pub fn display_schema(blueprint: &Blueprint) {
    Fmt::display(Fmt::heading(&"GraphQL Schema:\n".to_string()));
    let sdl = blueprint.to_schema();
    Fmt::display(format!("{}\n", print_schema::print_schema(sdl)));
}

fn display_config(config: &Config, n_plus_one_queries: bool) {
    let seq = vec![Fmt::n_plus_one_data(n_plus_one_queries, config)];
    Fmt::display(Fmt::table(seq));
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
