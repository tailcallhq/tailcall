use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall::core::generator::from_openapi_spec;

#[derive(Serialize, Deserialize)]
struct OpenAPIFixture {
    url: String,
    body: Value,
}

pub fn run_open_api_to_config_spec(path: &Path) -> datatest_stable::Result<()> {
    let name = path
        .file_name()
        .ok_or(anyhow::anyhow!("Invalid path"))?
        .to_str()
        .ok_or(anyhow::anyhow!("Invalid string"))?;
    let name = name.strip_suffix(".yml").unwrap();
    let content = fs::read_to_string(path)?;
    test_spec(name, content)?;
    Ok(())
}

fn test_spec(snapshot_name: &str, content: String) -> anyhow::Result<()> {
    let spec = oas3::from_reader(content.as_bytes())?;
    let config = from_openapi_spec("Query", spec);
    insta::assert_snapshot!(snapshot_name, config.to_sdl());
    Ok(())
}
