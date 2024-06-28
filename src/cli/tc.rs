use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use convert_case::{Case, Casing};
use dotenvy::dotenv;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use tailcall_macros::MergeRight;

use super::command::{Cli, Command};
use super::generator::Generator;
use super::update_checker;
use crate::cli::fmt::Fmt;
use crate::cli::runtime::{confirm_and_write, create_directory, select_prompt};
use crate::cli::server::Server;
use crate::cli::{self, CLIError};
use crate::core::blueprint::Blueprint;
use crate::core::config::reader::ConfigReader;
use crate::core::config::{Config, Expr, Field, RootSchema, Source, Type};
use crate::core::http::API_URL_PREFIX;
use crate::core::merge_right::MergeRight;
use crate::core::print_schema;
use crate::core::rest::{EndpointSet, Unchecked};
use crate::core::runtime::TargetRuntime;

const FILE_NAME: &str = ".tailcallrc.graphql";
const YML_FILE_NAME: &str = ".graphqlrc.yml";
const JSON_FILE_NAME: &str = ".tailcallrc.schema.json";

#[derive(Default, Deserialize, MergeRight, Serialize)]
struct GraphQLRC {
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    documents: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extensions: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    projects: Option<Value>,
}

lazy_static! {
    static ref TRACKER: tailcall_tracker::Tracker = tailcall_tracker::Tracker::default();
}
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
    match cli.command {
        Command::Start { file_paths } => {
            let config_module = config_reader.read_all(&file_paths).await?;
            log_endpoint_set(&config_module.extensions().endpoint_set);
            Fmt::log_n_plus_one(false, config_module.config());
            let server = Server::new(config_module);
            server.fork_start().await?;
            Ok(())
        }
        Command::Check { file_paths, n_plus_one_queries, schema, format } => {
            let config_module = (config_reader.read_all(&file_paths)).await?;
            log_endpoint_set(&config_module.extensions().endpoint_set);
            if let Some(format) = format {
                Fmt::display(format.encode(&config_module)?);
            }
            let blueprint = Blueprint::try_from(&config_module).map_err(CLIError::from);

            match blueprint {
                Ok(blueprint) => {
                    tracing::info!("Config {} ... ok", file_paths.join(", "));
                    Fmt::log_n_plus_one(n_plus_one_queries, config_module.config());
                    // Check the endpoints' schema
                    let _ = config_module
                        .extensions()
                        .endpoint_set
                        .clone()
                        .into_checked(&blueprint, runtime)
                        .await?;
                    if schema {
                        display_schema(&blueprint);
                    }
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
        Command::Init { folder_path } => init(runtime, &folder_path).await,
        Command::Gen { file_path } => {
            Generator::new(&file_path, runtime.clone())
                .generate()
                .await?;

            Ok(())
        }
    }
}

fn default_graphqlrc() -> GraphQLRC {
    GraphQLRC {
        schema: Some(Value::Sequence(vec!["./.tailcallrc.graphql".into()])),
        ..Default::default()
    }
}

async fn confirm_and_write_yml(
    runtime: TargetRuntime,
    yml_file_path: impl AsRef<Path>,
) -> Result<()> {
    let yml_file_path = yml_file_path.as_ref().display().to_string();

    let mut final_graphqlrc = default_graphqlrc();

    match runtime.file.read(yml_file_path.as_ref()).await {
        Ok(yml_content) => {
            let graphqlrc: GraphQLRC = serde_yaml::from_str(&yml_content)?;
            final_graphqlrc = graphqlrc.merge_right(final_graphqlrc);
            let content = serde_yaml::to_string(&final_graphqlrc)?;
            confirm_and_write(runtime.clone(), &yml_file_path, content.as_bytes()).await
        }
        Err(_) => {
            let content = serde_yaml::to_string(&final_graphqlrc)?;
            runtime.file.write(&yml_file_path, content.as_bytes()).await
        }
    }
}

pub async fn init(runtime: TargetRuntime, folder_path: &str) -> Result<()> {
    create_directory(folder_path).await?;

    let selection = select_prompt(
        "Please select the format in which you want to generate the config.",
        vec![Source::GraphQL, Source::Json, Source::Yml],
    )?;

    let tailcallrc = include_str!("../../generated/.tailcallrc.graphql");
    let tailcallrc_json: &str = include_str!("../../generated/.tailcallrc.schema.json");

    let file_path = Path::new(folder_path).join(FILE_NAME);
    let json_file_path = Path::new(folder_path).join(JSON_FILE_NAME);
    let yml_file_path = Path::new(folder_path).join(YML_FILE_NAME);

    confirm_and_write(
        runtime.clone(),
        &file_path.display().to_string(),
        tailcallrc.as_bytes(),
    )
    .await?;
    confirm_and_write(
        runtime.clone(),
        &json_file_path.display().to_string(),
        tailcallrc_json.as_bytes(),
    )
    .await?;
    confirm_and_write_yml(runtime.clone(), &yml_file_path).await?;
    create_main(runtime.clone(), folder_path, selection).await?;

    Ok(())
}

fn main_config() -> Config {
    let field = Field {
        type_of: "String".to_string(),
        required: true,
        const_field: Some(Expr { body: "Hello, World!".into() }),
        ..Default::default()
    };

    let query_type = Type {
        fields: BTreeMap::from([("greet".into(), field)]),
        ..Default::default()
    };

    Config {
        server: Default::default(),
        upstream: Default::default(),
        schema: RootSchema { query: Some("Query".to_string()), ..Default::default() },
        types: BTreeMap::from([("Query".into(), query_type)]),
        ..Default::default()
    }
}

async fn create_main(
    runtime: TargetRuntime,
    folder_path: impl AsRef<Path>,
    source: Source,
) -> Result<()> {
    let config = main_config();

    let content = match source {
        Source::GraphQL => config.to_sdl(),
        Source::Json => config.to_json(true)?,
        Source::Yml => config.to_yaml()?,
    };

    let path = folder_path
        .as_ref()
        .join(format!("main.{}", source.ext()))
        .display()
        .to_string();

    confirm_and_write(runtime.clone(), &path, content.as_bytes()).await?;
    Ok(())
}

fn log_endpoint_set(endpoint_set: &EndpointSet<Unchecked>) {
    let mut endpoints = endpoint_set.get_endpoints().clone();
    endpoints.sort_by(|a, b| {
        let method_a = a.get_method();
        let method_b = b.get_method();
        if method_a.eq(method_b) {
            a.get_path().as_str().cmp(b.get_path().as_str())
        } else {
            method_a.to_string().cmp(&method_b.to_string())
        }
    });
    for endpoint in endpoints {
        tracing::info!(
            "Endpoint: {} {}{} ... ok",
            endpoint.get_method(),
            API_URL_PREFIX,
            endpoint.get_path().as_str()
        );
    }
}

pub fn display_schema(blueprint: &Blueprint) {
    Fmt::display(Fmt::heading("GraphQL Schema:\n"));
    let sdl = blueprint.to_schema();
    Fmt::display(format!("{}\n", print_schema::print_schema(sdl)));
}
