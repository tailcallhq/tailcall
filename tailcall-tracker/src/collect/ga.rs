use http::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

use super::super::Result;
use super::Collect;
use crate::Event;

const GA_TRACKER_URL: &str = "https://www.google-analytics.com";

/// Event structure to be sent to GA
#[derive(Debug, Serialize, Deserialize)]
struct Payload {
    client_id: String,
    events: Vec<Event>,
}

impl Payload {
    pub fn new(event: Event) -> Self {
        Self { client_id: event.clone().client_id, events: vec![event] }
    }
}

pub struct Tracker {
    base_url: String,
    api_secret: String,
    measurement_id: String,
}

impl Tracker {
    pub fn new(api_secret: String, measurement_id: String) -> Self {
        Self {
            base_url: GA_TRACKER_URL.to_string(),
            api_secret,
            measurement_id,
        }
    }
    fn create_request(&self, event: Event) -> Result<reqwest::Request> {
        let event = Payload::new(event);
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
impl Collect for Tracker {
    async fn collect(&self, event: Event) -> Result<()> {
        let request = self.create_request(event)?;
        let client = reqwest::Client::new();
        client.execute(request).await?;

        Ok(())
    }
}
