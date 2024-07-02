use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall::core::generator::{Generator, Input};
use url::Url;

#[derive(Serialize, Deserialize)]
struct JsonFixture {
    url: String,
    body: Value,
}

datatest_stable::harness!(
    run_json_to_config_spec,
    "src/core/generator/tests/fixtures/json",
    r"^.*\.json"
);

pub fn run_json_to_config_spec(path: &Path) -> datatest_stable::Result<()> {
    let (url, body) = load_json(path)?;
    let parsed_url = Url::parse(url.as_str()).unwrap_or_else(|_| {
        panic!(
            "Failed to parse the url. url: {}, test file: {:?}",
            url, path
        )
    });
    test_spec(path, parsed_url, body)?;
    Ok(())
}

fn load_json(path: &Path) -> anyhow::Result<(String, Value)> {
    let contents = fs::read_to_string(path)?;
    let json_data: JsonFixture = serde_json::from_str(&contents).unwrap();
    Ok((json_data.url, json_data.body))
}

fn test_spec(path: &Path, url: Url, body: Value) -> anyhow::Result<()> {
    let config = Generator::default()
        .inputs(vec![Input::Json {
            url,
            response: body,
            field_name: "f1".to_string(),
        }])
        .generate(true)?;

    let snapshot_name = path.file_name().unwrap().to_str().unwrap();
    insta::assert_snapshot!(snapshot_name, config.to_sdl());
    Ok(())
}
