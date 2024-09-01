use chrono::{DateTime, Utc};
use reqwest::header::{HeaderName, HeaderValue};

use super::super::Result;
use crate::ga_event::GaEvent;
use crate::tracker::EventCollector;

const GA_TRACKER_URL: &str = "https://www.google-analytics.com";
const GA_TRACKER_API_SECRET: &str = "GVaEzXFeRkCI9YBIylbEjQ";
const GA_TRACKER_MEASUREMENT_ID: &str = "G-JEP3QDWT0G";

pub struct GaTracker {
    base_url: String,
    api_secret: String,
    measurement_id: String,
}

impl GaTracker {
    pub fn default() -> Self {
        Self {
            base_url: GA_TRACKER_URL.to_string(),
            api_secret: GA_TRACKER_API_SECRET.to_string(),
            measurement_id: GA_TRACKER_MEASUREMENT_ID.to_string(),
        }
    }
    fn create_request(
        &self,
        event_name: &str,
        start_time: DateTime<Utc>,
    ) -> Result<reqwest::Request> {
        let event = GaEvent::new(event_name, start_time);
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
}

#[async_trait::async_trait]
impl EventCollector for GaTracker {
    async fn dispatch(&self, event_name: &str, start_time: DateTime<Utc>) -> Result<()> {
        let request = self.create_request(event_name, start_time)?;
        let client = reqwest::Client::new();
        let response = client.execute(request).await?;
        let status = response.status();
        let text = response.text().await?;
        tracing::debug!("Collector: {}, message: {:?}", status.as_str(), text);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::EventCollector;

    #[tokio::test]
    async fn test_ga_tracker() {
        let ga_tracker = GaTracker::default();
        if let Err(e) = ga_tracker.dispatch("test", Utc::now()).await {
            panic!("Failed to dispatch event: {}", e);
        }
    }
}
