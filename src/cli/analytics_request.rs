use reqwest::{Client, header, StatusCode};
use serde_json::json;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;

const API_ENDPOINT: &str = "https://www.google-analytics.com/mp/collect";
const MEASUREMENT_ID: &str = "measurement_id_here";
const API_SECRET: &str = "api_secret_here";

lazy_static! {
    static ref URL: String = format!(
        "{}?api_secret={}&measurement_id={}",
        API_ENDPOINT, API_SECRET, MEASUREMENT_ID
    );
}

pub async fn ga_request(command: &str) -> Result<()> {
    const USER_AGENT: &str = "tailcall/1.0";

    let client = Client::new();
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_str(USER_AGENT)?);
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));

    let json_body = json!({
        "client_id": "1",
        "events": [
            {
                "name": "track_cli_usage",
                "params": {
                    "command_name": command
                }
            }
        ]
    });

    let response = client
        .post(&URL)
        .headers(headers)
        .json(&json_body)
        .send()
        .await?;

    if response.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(anyhow!("Failed to send analytics request"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    async fn test_ga_request_success() {
        let result = ga_request("start").await;
        assert!(result.is_ok());
    }
}