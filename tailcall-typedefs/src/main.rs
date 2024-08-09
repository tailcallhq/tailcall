mod gen_gql_schema;

use std::env;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use schemars::schema::{RootSchema, Schema};
use schemars::Map;
use serde_json::{json, Value};
use strum::IntoEnumIterator;
use tailcall::cli;
use tailcall::core::config::Config;
use tailcall::core::tracing::default_tracing_for_name;
use tailcall::core::{scalar, Error, FileIO};

static JSON_SCHEMA_FILE: &str = "../generated/.tailcallrc.schema.json";
static GRAPHQL_SCHEMA_FILE: &str = "../generated/.tailcallrc.graphql";

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

async fn mode_check() -> Result<(), Error> {
    let json_schema = get_file_path();
    let rt = cli::runtime::init(&Default::default());
    let file_io = rt.file;
    let content = file_io
        .read(json_schema.to_str().ok_or(Error::PathDeterminationFailed)?)
        .await?;
    let content = serde_json::from_str::<Value>(&content)?;
    let schema = get_updated_json().await?;
    match content.eq(&schema) {
        true => Ok(()),
        false => Err(Error::SchemaMismatch),
    }
}

async fn mode_fix() -> Result<()> {
    let rt = cli::runtime::init(&Default::default());
    let file_io = rt.file;

    update_json(file_io.clone()).await?;
    update_gql(file_io.clone()).await?;
    Ok(())
}

async fn update_gql(file_io: Arc<dyn FileIO>) -> Result<(), Error> {
    let doc = gen_gql_schema::build_service_document();

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GRAPHQL_SCHEMA_FILE);
    file_io
        .write(
            path.to_str().ok_or(anyhow!("Unable to determine path"))?,
            tailcall::core::document::print(doc).as_bytes(),
        )
        .await?;
    Ok(())
}

async fn update_json(file_io: Arc<dyn FileIO>) -> Result<(), Error> {
    let path = get_file_path();
    let schema = serde_json::to_string_pretty(&get_updated_json().await?)?;
    tracing::info!("Updating JSON Schema: {}", path.to_str().unwrap());
    file_io
        .write(
            path.to_str().ok_or(Error::PathDeterminationFailed)?,
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
    let scalar = scalar::Scalar::iter()
        .map(|scalar| (scalar.name(), scalar.schema()))
        .collect::<Map<String, Schema>>();
    schema.definitions.extend(scalar);

    let schema = json!(schema);
    Ok(schema)
}
