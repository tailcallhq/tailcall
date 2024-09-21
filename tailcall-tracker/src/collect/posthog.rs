use serde::de::Error;

use super::super::Result;
use super::Collect;
use crate::Event;

pub struct Tracker {
    api_secret: String,
}

impl Tracker {
    pub fn new(api_secret: String) -> Self {
        Self { api_secret }
    }
}

#[async_trait::async_trait]
impl Collect for Tracker {
    async fn collect(&self, event: Event) -> Result<()> {
        let api_secret = self.api_secret.clone();

        let handle_posthog = tokio::task::spawn_blocking(move || -> Result<()> {
            let client = posthog_rs::client(api_secret.as_str());
            let json = serde_json::to_value(&event)?;
            let mut posthog_event =
                posthog_rs::Event::new(event.event_name.clone(), event.client_id);

            match json {
                serde_json::Value::Object(map) => {
                    for (key, value) in map {
                        posthog_event.insert_prop(key, value)?;
                    }
                }
                _ => {
                    return Err(
                        serde_json::Error::custom("Failed to serialize event for posthog").into(),
                    );
                }
            }

            client.capture(posthog_event)?;
            Ok(())
        })
        .await;
        handle_posthog??;
        Ok(())
    }
}
