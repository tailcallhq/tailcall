use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall::core::generator::{Generator, Input, OperationType};
use url::Url;

#[derive(Serialize, Deserialize)]
struct JsonFixture {
    url: String,
    response: Value,
    body: Option<Value>,
}

datatest_stable::harness!(
    run_json_to_config_spec,
    "src/core/generator/tests/fixtures/json",
    r"^.*\.json"
);

pub fn run_json_to_config_spec(path: &Path) -> datatest_stable::Result<()> {
    let json_data = load_json(path)?;
    let parsed_url = Url::parse(json_data.url.as_str()).unwrap_or_else(|_| {
        panic!(
            "Failed to parse the url. url: {}, test file: {:?}",
            json_data.url, path
        )
    });
    test_spec(path, parsed_url, json_data)?;
    Ok(())
}

fn load_json(path: &Path) -> anyhow::Result<JsonFixture> {
    let contents = fs::read_to_string(path)?;
    let json_data: JsonFixture = serde_json::from_str(&contents).unwrap();
    Ok(json_data)
}

fn test_spec(path: &Path, url: Url, json_data: JsonFixture) -> anyhow::Result<()> {
    let cfg = if let Some(body) = json_data.body {
        Generator::default()
            .mutation(Some("Mutation".into()))
            .inputs(vec![Input::Json {
                url,
                response: json_data.response,
                field_name: "f1".to_string(),
                operation_type: OperationType::Mutation { body },
            }])
            .generate(true)?
    } else {
        Generator::default()
            .query(Some("Query".into()))
            .inputs(vec![Input::Json {
                url,
                response: json_data.response,
                field_name: "f1".to_string(),
                operation_type: OperationType::Query,
            }])
            .generate(true)?
    };

    let snapshot_name = path.file_name().unwrap().to_str().unwrap();
    insta::assert_snapshot!(snapshot_name, cfg.to_sdl());
    Ok(())
}
