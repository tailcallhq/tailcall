use serde::{Deserialize, Serialize};
use url::Url;

use crate::core::{
    config::{Config, ConfigModule},
    resource_reader::{Cached, ResourceReader},
    valid::Validator,
};

#[derive(Clone)]
pub struct SubGraphReader {
    reader: ResourceReader<Cached>,
}

#[derive(Serialize, Deserialize)]
struct AdminQuery {
    config: String,
}

impl SubGraphReader {
    pub fn new(reader: ResourceReader<Cached>) -> Self {
        Self { reader }
    }

    pub async fn fetch(&self, src: &str) -> anyhow::Result<ConfigModule> {
        let url = Url::parse(&format!("{src}/graphql"))?;

        let mut request = reqwest::Request::new(reqwest::Method::POST, url);

        let _ = request
            .body_mut()
            .insert(reqwest::Body::from(r#"{"query": "{ config }"}"#));

        let file_read = self.reader.read_file(request).await?;
        let admin_query: AdminQuery = serde_json::from_str(&file_read.content)?;

        let config = Config::from_sdl(&admin_query.config).to_result()?;
        let config_module = ConfigModule::from(config);

        Ok(config_module)
    }
}
