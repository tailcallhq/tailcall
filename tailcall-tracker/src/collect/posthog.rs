use chrono::{DateTime, Utc};
use tailcall_version::VERSION;

use super::super::Result;
use crate::helpers::{get_client_id, get_cpu_cores, get_os_name};
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
    async fn dispatch(&self, event_name: &str, start_time: DateTime<Utc>) -> Result<()> {
        let api_secret = self.api_secret.clone();
        let event_name = event_name.to_string();

        let handle_posthog = tokio::task::spawn_blocking(move || -> Result<()> {
            let client = posthog_rs::client(api_secret.as_str());
            let mut event = posthog_rs::Event::new(event_name.clone(), get_client_id());
            event.insert_prop("cpu_cores", get_cpu_cores())?;
            event.insert_prop("os_name", get_os_name())?;
            event.insert_prop("app_version", VERSION.as_str())?;
            event.insert_prop("start_time", start_time.to_string())?;
            if event_name == "ping" {
                let current_time = Utc::now();
                let uptime = current_time.signed_duration_since(start_time).num_minutes();
                event.insert_prop("uptime", format!("{} minutes", uptime))?;
            }
            client.capture(event)?;
            Ok(())
        })
        .await;
        handle_posthog??;
        Ok(())
    }
}
