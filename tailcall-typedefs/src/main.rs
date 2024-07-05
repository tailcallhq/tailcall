mod gen_gql_schema;

use std::env;
use std::path::PathBuf;
use std::process::exit;

use anyhow::{anyhow, Result};
use gen_gql_schema::update_gql;
use schemars::schema::{RootSchema, Schema};
use schemars::Map;
use serde_json::{json, Value};
use tailcall::cli;
use tailcall::core::config::Config;
use tailcall::core::scalar::CUSTOM_SCALARS;
use tailcall::core::tracing::default_tracing_for_name;

static JSON_SCHEMA_FILE: &str = "../generated/.tailcallrc.schema.json";

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(default_tracing_for_name("typedefs")).unwrap();
    let args: Vec<String> = env::args().collect();
    let arg = args.get(1);

    if arg.is_none() {
        tracing::error!("An argument required, you can pass either `fix` or `check` argument");
        return;
    }
    match arg.unwrap().as_str() {
        "fix" => {
            let result = mode_fix().await;
            if let Err(e) = result {
                tracing::error!("{}", e);
                exit(1);
            }
        }
        "check" => {
            let result = mode_check().await;
            if let Err(e) = result {
                tracing::error!("{}", e);
                exit(1);
            }
        }
        &_ => {
            tracing::error!("Unknown argument, you can pass either `fix` or `check` argument");
            return;
        }
    }
}

async fn mode_check() -> Result<()> {
    let json_schema = get_file_path();
    let rt = cli::runtime::init(&Default::default());
    let file_io = rt.file;
    let content = file_io
        .read(
            json_schema
                .to_str()
                .ok_or(anyhow!("Unable to determine path"))?,
        )
        .await?;
    let content = serde_json::from_str::<Value>(&content)?;
    let schema = get_updated_json().await?;
    match content.eq(&schema) {
        true => Ok(()),
        false => Err(anyhow!("Schema mismatch")),
    }
}

async fn mode_fix() -> Result<()> {
    update_json().await?;
    update_gql()?;
    Ok(())
}

async fn update_json() -> Result<()> {
    let path = get_file_path();
    let schema = serde_json::to_string_pretty(&get_updated_json().await?)?;
    let rt = cli::runtime::init(&Default::default());
    let file_io = rt.file;
    tracing::info!("Updating JSON Schema: {}", path.to_str().unwrap());
    file_io
        .write(
            path.to_str().ok_or(anyhow!("Unable to determine path"))?,
            schema.as_bytes(),
        )
        .await?;
    Ok(())
}

fn get_file_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(JSON_SCHEMA_FILE)
}

async fn get_updated_json() -> Result<Value> {
    let mut schema: RootSchema = schemars::schema_for!(Config);
    let scalar = CUSTOM_SCALARS
        .iter()
        .map(|(k, v)| (k.clone(), v.schema()))
        .collect::<Map<String, Schema>>();
    schema.definitions.extend(scalar);

    let schema = json!(schema);
    Ok(schema)
}
