use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall::core::generator::{Generator, Input};
use tailcall::core::http::Method;
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct APIRequest {
    #[serde(default)]
    pub method: Method,
    pub url: Url,
    #[serde(default)]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(default, rename = "body")]
    pub body: Option<Value>,
}

mod default {
    pub fn status() -> u16 {
        200
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct APIResponse {
    #[serde(default = "default::status")]
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default, rename = "body")]
    pub body: Option<Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsonFixture {
    request: APIRequest,
    response: APIResponse,
    #[serde(default)]
    is_mutation: Option<bool>,
    field_name: String,
}

datatest_stable::harness!(
    run_json_to_config_spec,
    "src/core/generator/tests/fixtures/json",
    r"^.*\.json"
);

pub fn run_json_to_config_spec(path: &Path) -> datatest_stable::Result<()> {
    let json_data = load_json(path)?;
    test_spec(path, json_data)?;
    Ok(())
}

fn load_json(path: &Path) -> anyhow::Result<JsonFixture> {
    let contents = fs::read_to_string(path)?;
    let json_data: JsonFixture = serde_json::from_str(&contents).unwrap();
    Ok(json_data)
}

fn test_spec(path: &Path, json_data: JsonFixture) -> anyhow::Result<()> {
    let JsonFixture { request, response, is_mutation, field_name } = json_data;

    let req_body = request.body.unwrap_or_default();
    let resp_body = response.body.unwrap_or_default();

    let generator = Generator::default().inputs(vec![Input::Json {
        url: request.url,
        method: request.method,
        req_body,
        res_body: resp_body,
        field_name,
        is_mutation: is_mutation.unwrap_or_default(),
        headers: request.headers,
    }]);

    let cfg = if is_mutation.unwrap_or_default() {
        generator.mutation(Some("Mutation".into()))
    } else {
        generator
    }
    .generate(true)?;

    let snapshot_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid snapshot name"))?;

    insta::assert_snapshot!(snapshot_name, cfg.to_sdl());
    Ok(())
}
