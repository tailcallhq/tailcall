use crate::helpers::{get_client_id, get_cpu_cores, get_os_name};

use super::super::Result;

use crate::tracker::EventCollector;

const POSTHOG_API_KEY: &str = "phc_CWdKhSxlSsKceZhhnSJ1LfkaGxgYhZLh4Fx7ssjrkRf";

pub struct PostHogTracker {
    api_secret: String,
}

impl PostHogTracker {
    pub fn default() -> Self {
        Self { api_secret: POSTHOG_API_KEY.to_string() }
    }
}

#[async_trait::async_trait]
impl EventCollector for PostHogTracker {
    async fn dispatch(&self, event_name: &str) -> Result<()> {
        let api_secret = self.api_secret.clone();
        let event_name = event_name.to_string();

        tokio::task::spawn_blocking(move || {
            let client = posthog_rs::client(api_secret.as_str());
            let mut event = posthog_rs::Event::new(get_client_id(), event_name);
            event.insert_prop("cpu_cores", get_cpu_cores()).unwrap();
            event.insert_prop("os_name", get_os_name()).unwrap();
            client.capture(event).unwrap();
        })
        .await
        .unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::EventCollector;

    #[tokio::test]
    async fn test_posthog_tracker() {
        let posthog_tracker = PostHogTracker::default();
        if let Err(e) = posthog_tracker.dispatch("test").await {
            panic!("Failed to dispatch event: {}", e);
        }
    }
}
