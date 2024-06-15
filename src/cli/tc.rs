use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use convert_case::{Case, Casing};
use dotenvy::dotenv;
use inquire::{Confirm, Select, Text};
use lazy_static::lazy_static;

use super::command::{Cli, Command};
use super::update_checker;
use crate::cli::fmt::Fmt;
use crate::cli::server::Server;
use crate::cli::{self, CLIError};
use crate::core::blueprint::Blueprint;
use crate::core::config::reader::ConfigReader;
use crate::core::generator::Generator;
use crate::core::http::API_URL_PREFIX;
use crate::core::print_schema;
use crate::core::rest::{EndpointSet, Unchecked};
const FILE_NAME: &str = ".tailcallrc.graphql";
const YML_FILE_NAME: &str = ".graphqlrc.yml";
const JSON_FILE_NAME: &str = ".tailcallrc.schema.json";

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
        Command::Init { folder_path } => {
            init(&folder_path).await?;
            Ok(())
        }
        Command::Gen { paths, input, output, query } => {
            let generator = Generator::init(runtime);
            let cfg = generator
                .read_all(input, paths.as_ref(), query.as_str())
                .await?;

            let config = output.unwrap_or_default().encode(&cfg)?;
            Fmt::display(config);
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

    // Prompt for project details
    let project_name = Text::new("Project Name:")
        .with_default("my-app")
        .prompt()
        .context("Failed to prompt for project name")?;

    let file_formats = vec!["GraphQL", "JSON", "YML"];
    let file_format = Select::new("File Format:", file_formats)
        .prompt()
        .context("Failed to prompt for file format")?;

    // Determine file paths based on selected format
    let (config_directory, config_file_name) = match file_format {
        "GraphQL" => ("config", FILE_NAME),
        "JSON" => ("config", JSON_FILE_NAME),
        "YML" => (".", YML_FILE_NAME),
        _ => unreachable!(), // This should never happen due to the Select prompt
    };

    // Summary and confirmation
    println!("Creating the following project structure:");
    println!("- {}/src/main.rs", project_name);
    println!(
        "- {}/{}/{}",
        project_name, config_directory, config_file_name
    );

    let confirm = Confirm::new("Is this OK?")
        .with_default(true)
        .prompt()
        .context("Failed to confirm project initialization")?;

    if confirm {
        // Create project directories and files
        let project_path = Path::new(folder_path).join(&project_name);
        let src_path = project_path.join("src");
        let config_path = project_path.join(config_directory);

        fs::create_dir_all(&src_path)
            .with_context(|| format!("Failed to create directory {:?}", &src_path))?;
        fs::create_dir_all(&config_path)
            .with_context(|| format!("Failed to create directory {:?}", &config_path))?;

        // Create main.rs with Hello, World! GraphQL server
        let main_rs_path = src_path.join("main.rs");
        let mut main_rs_file = File::create(&main_rs_path)
            .with_context(|| format!("Failed to create file {:?}", &main_rs_path))?;

        writeln!(main_rs_file, r#"
use async_graphql::Context, Object, Schema;
use async_std::task;

#[derive(Default)]
struct Query;

#[Object]
impl Query {{
    async fn hello(&self, _ctx: &Context<'_>) -> String {{
        "Hello, World!".to_string()
    }}
}}

#[tokio::main]
async fn main() {{
    let schema = Schema::build(Query::default(), async_graphql::EmptyMutation, async_graphql::EmptySubscription).finish();
    let addr = "127.0.0.1:8000".parse().unwrap();
    let _ = async_graphql_actix_web::run(schema, addr).await.unwrap();
}}
"#).with_context(|| "Failed to write to main.rs file")?;

        // Create configuration file with initial content
        let config_file_path = config_path.join(config_file_name);
        let mut config_file = File::create(&config_file_path)
            .with_context(|| format!("Failed to create file {:?}", &config_file_path))?;

        // Write initial content based on selected format
        match file_format {
            "GraphQL" => {
                let tailcallrc_graphql = include_str!("../../generated/.tailcallrc.graphql");
                writeln!(config_file, "{}", tailcallrc_graphql)
                    .with_context(|| "Failed to write GraphQL configuration file")?;
            }
            "JSON" => {
                let tailcallrc_json = include_str!("../../generated/.tailcallrc.schema.json");
                writeln!(config_file, "{}", tailcallrc_json)
                    .with_context(|| "Failed to write JSON configuration file")?;
            }
            "YML" => {
                let graphqlrc_yml = include_str!("../../.graphqlrc.yml");
                writeln!(config_file, "{}", graphqlrc_yml)
                    .with_context(|| "Failed to write YAML configuration file")?;
            }
            _ => {}
        }

        println!("Project initialized successfully.");
    } else {
        println!("Initialization cancelled.");
    }

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
