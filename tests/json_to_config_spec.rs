use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall::core::generator::{from_json, ConfigGenerationRequest};


#[derive(Serialize, Deserialize)]
struct JsonFixture {
    url: String,
    body: Value,
}


pub fn run_json_to_config_spec(path: &Path) -> datatest_stable::Result<()> {
    let (url, body) = load_json(path)?;
    test_spec(path, url, body)?;
    Ok(())
}

fn load_json(path: &Path) -> anyhow::Result<(String, Value)> {
    let contents = fs::read_to_string(path)?;
    let json_data: JsonFixture = serde_json::from_str(&contents).unwrap();
    Ok((json_data.url, json_data.body))
}

fn test_spec(path: &Path, url: String, body: Value) -> anyhow::Result<()> {
    let config = from_json(&[ConfigGenerationRequest::new(&url, &body)])?;
    let snapshot_name = path.file_name().unwrap().to_str().unwrap();
    insta::assert_snapshot!(snapshot_name, config.to_sdl());
    Ok(())
}
