use tailcall_version::VERSION;

use super::{super::Result, collectors::EventCollector};
use crate::tracker::Event;

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
    async fn dispatch(&self, event: Event) -> Result<()> {
        let api_secret = self.api_secret.clone();

        let handle_posthog = tokio::task::spawn_blocking(move || -> Result<()> {
            let client = posthog_rs::client(api_secret.as_str());
            let mut posthog_event =
                posthog_rs::Event::new(event.event_name.clone(), event.client_id);
            posthog_event.insert_prop("cpu_cores", event.cores)?;
            posthog_event.insert_prop("os_name", event.os_name)?;
            posthog_event.insert_prop("app_version", VERSION.as_str())?;
            posthog_event.insert_prop("start_time", event.start_time)?;
            if let Some(args) = event.args {
                posthog_event.insert_prop("args", args.join(", "))?;
            }
            if let Some(uptime) = event.up_time {
                posthog_event.insert_prop("uptime", uptime)?;
            }
            if let Some(path) = event.path {
                posthog_event.insert_prop("path", path)?;
            }
            if let Some(user) = event.user {
                posthog_event.insert_prop("user", user)?;
            }
            client.capture(posthog_event)?;
            Ok(())
        })
        .await;
        handle_posthog??;
        Ok(())
    }
}
