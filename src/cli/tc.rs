use std::fs;
use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use convert_case::{Case, Casing};
use dotenvy::dotenv;
use inquire::Confirm;
use lazy_static::lazy_static;
use stripmargin::StripMargin;

use super::command::{Cli, Command};
use super::update_checker;
use crate::cli::fmt::Fmt;
use crate::cli::server::Server;
use crate::cli::{self, CLIError};
use crate::core::blueprint::Blueprint;
use crate::core::config::reader::ConfigReader;
use crate::core::config::{Resolution, ResolveOptions};
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
    let cli = Cli::parse();
    let writer = std::io::stdout().lock();
    run_inner(cli, writer).await
}

pub async fn run_inner<W: std::io::Write>(cli: Cli, mut writer: W) -> Result<()> {
    if let Ok(path) = dotenv() {
        tracing::info!("Env file: {:?} loaded", path);
    }
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
                Fmt::display(format.encode(&config_module)?, &mut writer);
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
                        display_schema(&blueprint, &mut writer);
                    }
                    Ok(())
                }
                Err(e) => Err(e.into()),
            }
        }
        Command::Init { folder_path } => init(&folder_path).await,
        Command::Gen {
            file_paths,
            input,
            output,
            query,
            resolve_ambiguous_input,
            resolve_ambiguous_output,
        } => {
            let generator = Generator::init(runtime);
            let (resolve_ambiguous_input, resolve_ambiguous_output) =
                validate_resolutions(resolve_ambiguous_input, resolve_ambiguous_output)?;

            let cfg = generator
                .read_all(
                    input,
                    file_paths.as_ref(),
                    query.as_str(),
                    resolver_fn(resolve_ambiguous_input, resolve_ambiguous_output),
                )
                .await?;

            let config = output.unwrap_or_default().encode(&cfg)?;
            Fmt::display(config, &mut writer);
            Ok(())
        }
    }
}

fn resolver_fn(
    resolve_ambiguous_input: Option<ResolveOptions>,
    resolve_ambiguous_output: Option<ResolveOptions>,
) -> impl Fn(&str) -> Resolution {
    move |v| {
        let mut resolution =
            Resolution { input: format!("IN_{}", v), output: format!("OUT_{}", v) };
        if let Some(ResolveOptions { prefix, suffix }) = &resolve_ambiguous_input {
            resolution.input = format!("{}{}{}", prefix, v, suffix);
        }

        if let Some(ResolveOptions { prefix, suffix }) = &resolve_ambiguous_output {
            resolution.output = format!("{}{}{}", prefix, v, suffix);
        }
        resolution
    }
}

fn validate_resolutions(
    input: Option<String>,
    output: Option<String>,
) -> Result<(Option<ResolveOptions>, Option<ResolveOptions>)> {
    let input = input
        .as_ref()
        .and_then(|v| ResolveOptions::from_str(v).ok());

    let output = output
        .as_ref()
        .and_then(|v| ResolveOptions::from_str(v).ok());

    if let Some(input) = input.as_ref() {
        if let Some(output) = output.as_ref() {
            validate_against(input, output)?;
        }
    }

    Ok((input, output))
}

// TODO move this to impl ResolveOptions once it's integrated with
// *Reader::read_all
pub fn validate_against(s: &ResolveOptions, other: &ResolveOptions) -> Result<()> {
    let lhs = format!("{}{}", s.prefix, s.suffix);
    let rhs = format!("{}{}", other.prefix, other.suffix);

    if lhs == rhs {
        return Err(anyhow::anyhow!(
            "Input and output resolutions cannot be the same"
        ));
    }
    Ok(())
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
    let tailcallrc_json: &str = include_str!("../../generated/.tailcallrc.schema.json");

    let file_path = Path::new(folder_path).join(FILE_NAME);
    let json_file_path = Path::new(folder_path).join(JSON_FILE_NAME);
    let yml_file_path = Path::new(folder_path).join(YML_FILE_NAME);

    let tailcall_exists = fs::metadata(&file_path).is_ok();

    if tailcall_exists {
        // confirm overwrite
        let confirm = Confirm::new(&format!("Do you want to overwrite the file {}?", FILE_NAME))
            .with_default(false)
            .prompt()?;

        if confirm {
            fs::write(&file_path, tailcallrc.as_bytes())?;
            fs::write(&json_file_path, tailcallrc_json.as_bytes())?;
        }
    } else {
        fs::write(&file_path, tailcallrc.as_bytes())?;
        fs::write(&json_file_path, tailcallrc_json.as_bytes())?;
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

pub fn display_schema<W: std::io::Write>(blueprint: &Blueprint, writer: &mut W) {
    Fmt::display(Fmt::heading("GraphQL Schema:\n"), writer);
    let sdl = blueprint.to_schema();
    Fmt::display(format!("{}\n", print_schema::print_schema(sdl)), writer);
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use clap::Parser;

    use crate::cli::command::Cli;

    #[tokio::test]
    async fn test_run_inner() {
        let args = [
            "tailcall",
            "gen",
            "tailcall-fixtures/fixtures/generator/proto/news.proto",
            "--input",
            "proto",
            "--output",
            "graphql",
            "--resolve-ambiguous-input",
            "prefix=InPrefix_",
            "--resolve-ambiguous-output",
            "prefix=OutPrefix_",
        ];
        let cli = Cli::parse_from(args);
        let mut cursor = Cursor::new(Vec::new());
        super::run_inner(cli, &mut cursor).await.unwrap();
        let output = String::from_utf8(cursor.into_inner()).unwrap();
        insta::assert_snapshot!(output);
    }
}
