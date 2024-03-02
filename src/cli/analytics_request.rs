use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

// secrets & constant
const API_SECRET: &str = "secret";
const MEASUREMENT_ID: &str = "G-NCQKBRNVDW";
const BASE_URL: &str = "https://www.google-analytics.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct Params {
    #[serde(rename = "command_name")]
    pub command_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub params: Params,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostData {
    #[serde(rename = "client_id")]
    pub client_id: String,
    pub events: Vec<Event>,
}

pub struct ApiClient {
    client: Client,
    base_url: String,
    api_secret: String,
    measurement_id: String,
}

impl ApiClient {
    pub fn new() -> Self {
        let client = Client::new();
        ApiClient {
            client,
            base_url: BASE_URL.trim_end_matches('/').to_string(),
            api_secret: API_SECRET.to_string(),
            measurement_id: MEASUREMENT_ID.to_string(),
        }
    }

    pub async fn post_data(&self, command_name: &str) -> Result<String, anyhow::Error> {
        let post_data = PostData {
            client_id: "1".to_string(),
            events: vec![Event {
                name: "track_cli_usage".to_string(),
                params: Params { command_name: command_name.to_string() },
            }],
        };

        let json_data =
            serde_json::to_string(&post_data).with_context(|| "Failed to serialize data")?;

        let response = self
            .client
            .post(format!("{}/debug/mp/collect", self.base_url))
            .query(&[
                ("api_secret", &self.api_secret),
                ("measurement_id", &self.measurement_id),
            ])
            .header("Content-Type", "application/json")
            .body(json_data.clone())
            .send()
            .await
            .with_context(|| "Failed to send request")?;

        let response_text = response
            .text()
            .await
            .with_context(|| "Failed to read response")?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .with_context(|| "Failed to parse response JSON")?;

        let validation_messages = response_json.get("validationMessages");

        if validation_messages
            .as_ref()
            .map(|v| v.as_array().map_or(true, Vec::is_empty))
            .unwrap_or_default()
        {
            self.client
                .post(format!("{}/mp/collect", self.base_url))
                .query(&[
                    ("api_secret", &self.api_secret),
                    ("measurement_id", &self.measurement_id),
                ])
                .header("Content-Type", "application/json")
                .body(json_data)
                .send()
                .await?;

            Ok("Success!".to_string())
        } else {
            Err(anyhow::Error::msg("API request failed"))
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use super::*;

    #[test]
    async fn test_new_api_client() {
        let api_client = ApiClient::new();

        assert_eq!(api_client.base_url, BASE_URL.trim_end_matches('/'));
        assert_eq!(api_client.api_secret, API_SECRET);
        assert_eq!(api_client.measurement_id, MEASUREMENT_ID);
    }

    /* TODO: success case
    #[test]
    async fn test_post_data_success() {

    }*/
}
