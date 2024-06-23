use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use convert_case::{Case, Casing};
use dotenvy::dotenv;
use inquire::{Confirm, Select};
use lazy_static::lazy_static;
use stripmargin::StripMargin;

use super::command::{Cli, Command};
use super::generator::Generator;
use super::update_checker;
use crate::cli::fmt::Fmt;
use crate::cli::server::Server;
use crate::cli::{self, CLIError};
use crate::core::blueprint::Blueprint;
use crate::core::config::reader::ConfigReader;
use crate::core::config::{Config, Expr, Field, RootSchema, Type};
use crate::core::http::API_URL_PREFIX;
use crate::core::print_schema;
use crate::core::rest::{EndpointSet, Unchecked};
use crate::core::runtime::TargetRuntime;

const FILE_NAME: &str = ".tailcallrc.graphql";
const YML_FILE_NAME: &str = ".graphqlrc.yml";
const JSON_FILE_NAME: &str = ".tailcallrc.schema.json";

const GRAPHQL: &str = "GraphQL";
const JSON: &str = "JSON";
const YML: &str = "YML";

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
            log_endpoint_set(&config_module.extensions.endpoint_set);
            Fmt::log_n_plus_one(false, &config_module.config);
            let server = Server::new(config_module);
            server.fork_start().await?;
            Ok(())
        }
        Command::Check { file_paths, n_plus_one_queries, schema, format } => {
            let config_module = (config_reader.read_all(&file_paths)).await?;
            log_endpoint_set(&config_module.extensions.endpoint_set);
            if let Some(format) = format {
                Fmt::display(format.encode(&config_module)?);
            }
            let blueprint = Blueprint::try_from(&config_module).map_err(CLIError::from);

            match blueprint {
                Ok(blueprint) => {
                    tracing::info!("Config {} ... ok", file_paths.join(", "));
                    Fmt::log_n_plus_one(n_plus_one_queries, &config_module.config);
                    // Check the endpoints' schema
                    let _ = config_module
                        .extensions
                        .endpoint_set
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

async fn confirm_overwrite(
    runtime: TargetRuntime,
    file_path: impl AsRef<Path>,
    content: &[u8],
) -> Result<()> {
    let file_exists = fs::metadata(file_path.as_ref()).is_ok();

    if file_exists {
        // confirm overwrite
        let confirm = Confirm::new(&format!(
            "Do you want to overwrite the file {}?",
            file_path.as_ref().display()
        ))
        .with_default(false)
        .prompt()?;

        if !confirm {
            return Ok(());
        }
    }

    runtime
        .file
        .write(&file_path.as_ref().display().to_string(), content)
        .await?;

    Ok(())
}

async fn confirm_overwrite_yml(
    runtime: TargetRuntime,
    file_path: impl AsRef<Path>,
    yml_file_path: impl AsRef<Path>,
) -> Result<()> {
    let yml_exists = fs::metadata(&yml_file_path).is_ok();
    let yml_path_str = yml_file_path.as_ref().display().to_string();

    if !yml_exists {
        let graphqlrc = r"|schema:
         |- './.tailcallrc.graphql'
    "
        .strip_margin();

        runtime
            .file
            .write(yml_path_str.as_str(), graphqlrc.as_bytes())
            .await?;
    }

    let graphqlrc = fs::read_to_string(&yml_file_path)?;

    let file_path = file_path.as_ref().to_str().unwrap();

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
                    runtime
                        .file
                        .write(yml_path_str.as_str(), updated.as_bytes())
                        .await?;
                }
            }
        }
    }

    Ok(())
}

/// Checks if file or folder already exists or not.
fn is_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

pub async fn init(runtime: TargetRuntime, folder_path: &str) -> Result<()> {
    let folder_exists = is_exists(folder_path);

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

    let selection = Select::new(
        "Please select the format in which you want to generate the config.",
        vec![GRAPHQL, JSON, YML],
    )
    .prompt()?;

    let tailcallrc = include_str!("../../generated/.tailcallrc.graphql");
    let tailcallrc_json: &str = include_str!("../../generated/.tailcallrc.schema.json");

    let file_path = Path::new(folder_path).join(FILE_NAME);
    let json_file_path = Path::new(folder_path).join(JSON_FILE_NAME);
    let yml_file_path = Path::new(folder_path).join(YML_FILE_NAME);

    match selection {
        GRAPHQL => {
            confirm_overwrite(runtime.clone(), &file_path, tailcallrc.as_bytes()).await?;
            create_main(runtime.clone(), folder_path, "graphql").await?;
        }
        JSON => {
            confirm_overwrite(runtime.clone(), &json_file_path, tailcallrc_json.as_bytes()).await?;
            create_main(runtime.clone(), folder_path, "json").await?;
        }
        YML => {
            confirm_overwrite(runtime.clone(), &file_path, tailcallrc.as_bytes()).await?;
            confirm_overwrite_yml(runtime.clone(), &file_path, &yml_file_path).await?;
            create_main(runtime.clone(), folder_path, "yml").await?;
        }
        _ => {
            unreachable!()
        }
    }

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
    extension: &str,
) -> Result<()> {
    let config = main_config();

    let content = match extension {
        "graphql" => config.to_sdl(),
        "json" => config.to_json(true)?,
        "yml" => config.to_yaml()?,
        _ => {
            unreachable!()
        }
    };

    let path = folder_path
        .as_ref()
        .join(format!("main.{}", extension))
        .display()
        .to_string();
    runtime
        .file
        .write(path.as_str(), content.as_bytes())
        .await?;
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
