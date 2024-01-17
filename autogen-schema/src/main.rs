use std::env;
use std::fs::File;
use std::path::PathBuf;
use anyhow::Result;
use env_logger::Env;

fn logger_init() {
    let env = Env::new();
    env_logger::Builder::from_env(env).init();
}

fn main() {
    logger_init();

    match update_json_schema() {
        Ok(_) => {
            log::info!("Json Schema updated successfully.")
        },
        Err(e) => {
            log::error!("Unable to update json schema due to: {}", e)
        }
    }
}

fn update_json_schema() -> Result<()> {
    let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root_dir.pop();
    root_dir.push("examples");

    let mut json_placeholder = root_dir.clone();
    json_placeholder.push("jsonplaceholder.json");

    // let mut json_placeholder_gql = root_dir.clone();
    // json_placeholder_gql.push("jsonplaceholder.graphql");

    let mut json_schema = root_dir.clone();
    json_schema.push(JSON_SCHEMA_FILE);

    let mut gql_schema = root_dir;
    gql_schema.push(GQL_SCHEMA_FILE);

    let mut file = File::open(json_placeholder)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let value = serde_json::from_str::<Value>(&content)?;
    let schema_json = schemars::schema_for_value!(value);
    let schema_json = serde_json::to_string_pretty(&schema_json)?;

    // let mut file = File::open(json_placeholder_gql)?;
    // let mut content = String::new();
    // file.read_to_string(&mut content)?;

    // let value = serde_json::from_str::<async_graphql::Value>(&content)?;
    // let schema_gql = schemars::schema_for_value!(value);
    // let schema_gql = serde_json::to_string_pretty(&schema_gql)?;

    let mut file = File::create(json_schema)?;
    file.write_all(schema_json.as_bytes())?;

    // let mut file = File::create(gql_schema)?;
    // file.write_all(schema_gql.as_bytes())?;

    Ok(())
}