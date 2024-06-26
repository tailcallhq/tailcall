use reqwest::header::{HeaderName, HeaderValue};

use super::Result;
use crate::check_tracking::check_tracking;
use crate::event::Event;

const API_SECRET: &str = "GVaEzXFeRkCI9YBIylbEjQ";
const MEASUREMENT_ID: &str = "G-JEP3QDWT0G";
const BASE_URL: &str = "https://www.google-analytics.com";

///
/// Base structure to track usage of the CLI application
#[derive(Debug, Clone)]
pub struct Tracker {
    base_url: String,
    api_secret: String,
    measurement_id: String,
    is_tracking: bool,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            base_url: BASE_URL.to_string(),
            api_secret: API_SECRET.to_string(),
            measurement_id: MEASUREMENT_ID.to_string(),
            is_tracking: check_tracking(),
        }
    }
}

impl Tracker {
    /// Initializes the ping event to be sent after the provided duration
    pub async fn init_ping(&'static self, duration: tokio::time::Duration) {
        if self.is_tracking {
            let mut interval = tokio::time::interval(duration);
            tokio::task::spawn(async move {
                loop {
                    interval.tick().await;
                    let _ = self.dispatch("ping").await;
                }
            });
        }
    }

    fn create_request(&self, event_name: &str) -> Result<reqwest::Request> {
        let event = Event::new(event_name);
        tracing::debug!("Sending event: {:?}", event);
        let mut url = reqwest::Url::parse(self.base_url.as_str())?;
        url.set_path("/mp/collect");
        url.query_pairs_mut()
            .append_pair("api_secret", self.api_secret.as_str())
            .append_pair("measurement_id", self.measurement_id.as_str());
        let mut request = reqwest::Request::new(reqwest::Method::POST, url);
        let header_name = HeaderName::from_static("content-type");
        let header_value = HeaderValue::from_str("application/json")?;
        request.headers_mut().insert(header_name, header_value);

        let _ = request
            .body_mut()
            .insert(reqwest::Body::from(serde_json::to_string(&event)?));
        Ok(request)
    }

    pub async fn dispatch(&'static self, name: &str) -> Result<()> {
        if self.is_tracking {
            let request = self.create_request(name)?;
            let client = reqwest::Client::new();
            let response = client.execute(request).await?;
            let status = response.status();
            let text = response.text().await?;
            tracing::debug!("Tracker: {}, message: {:?}", status.as_str(), text);
        }

        Ok(())
    }
}
