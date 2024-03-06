use anyhow::{Context, Result};
use lazy_static::lazy_static;
use reqwest::Client;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref API_SECRET: String = "secret".to_string();
    static ref MEASUREMENT_ID: String = "G-NCQKBRNVDW".to_string();
    static ref BASE_URL: String = "https://www.google-analytics.com".to_string();
}

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

    pub async fn post_data(&self, command_name: &str) -> Result<(), anyhow::Error> {
        self.validate_and_send_data(command_name).await?;
        Ok(())
    }

    pub async fn validate_and_send_data(
        &self,
        command_name: &str,
    ) -> Result<String, anyhow::Error> {
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
    use serde_json::json;

    use super::*;

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    #[tokio::test]
    async fn test_validate_and_send_data_success() {
        let server = start_mock_server();
        let client = ApiClient::new();
        let m = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/debug/mp/collect")
                .query_param("api_secret", &client.api_secret)
                .query_param("measurement_id", &client.measurement_id)
                .json_body(json!({
                    "client_id": "1",
                    "events": [
                        {
                            "name": "track_cli_usage",
                            "params": {
                                "command_name": "start"
                            }
                        }
                    ]
                }));
            then.status(200).json_body(json!({
                "validationMessages": []
            }));
        });

        let uri = format!("http://{}/debug/mp/collect", m.server_address());
        let response = reqwest::Client::new()
            .post(uri)
            .query(&[("api_secret", &client.api_secret), ("measurement_id", &client.measurement_id)])
            .header("Content-Type", "application/json")
            .body(r#"{"client_id":"1","events":[{"name":"track_cli_usage","params":{"command_name":"start"}}]}"#)
            .send()
            .await
            .unwrap();

        m.assert();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_validate_and_send_data_json_payload_error() {
        let server = start_mock_server();
        let client = ApiClient::new();
        let m = server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/debug/mp/collect")
                .query_param("api_secret", &client.api_secret)
                .query_param("measurement_id", &client.measurement_id)
                .json_body(json!({
                    "client_id": "1",
                    "events": [
                        {
                            "name": "+track_cli_usage",
                            "params": {
                                "command_name": "start"
                            }
                        }
                    ]
                }));
            then.status(200)
                .json_body(json!({
                    "validationMessages": [
                      {
                        "fieldPath": "events",
                        "description": "Event at index: [0] has invalid name [+track_cli_usage]. Names must start with an alphabetic character.",
                        "validationCode": "NAME_INVALID"
                      }
                    ]
                  }));
        });

        let uri = format!("http://{}/debug/mp/collect", m.server_address());
        let response = reqwest::Client::new()
            .post(uri)
            .query(&[("api_secret", &client.api_secret), ("measurement_id", &client.measurement_id)])
            .header("Content-Type", "application/json")
            .body(r#"{"client_id":"1","events":[{"name":"+track_cli_usage","params":{"command_name":"start"}}]}"#)
            .send()
            .await
            .unwrap();

        m.assert();
        assert_eq!(response.status(), 200);
    }
}
