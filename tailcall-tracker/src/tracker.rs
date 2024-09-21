use chrono::{DateTime, Utc};
use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
use sysinfo::System;
use tokio::time::Duration;

use super::Result;
use crate::can_track::can_track;
use crate::collect::{ga, posthog, Collect};
use crate::{Event, EventKind};

const GA_TRACKER_API_SECRET: &str = match option_env!("GA_API_SECRET") {
    Some(val) => val,
    None => "dev",
};
const GA_TRACKER_MEASUREMENT_ID: &str = match option_env!("GA_MEASUREMENT_ID") {
    Some(val) => val,
    None => "dev",
};
const POSTHOG_API_SECRET: &str = match option_env!("POSTHOG_API_SECRET") {
    Some(val) => val,
    None => "dev",
};

const PARAPHRASE: &str = "tc_key";

const DEFAULT_CLIENT_ID: &str = "<anonymous>";

pub struct Tracker {
    collectors: Vec<Box<dyn Collect>>,
    is_tracking: bool,
    start_time: DateTime<Utc>,
}

impl Default for Tracker {
    fn default() -> Self {
        let ga_tracker = Box::new(ga::Tracker::new(
            GA_TRACKER_API_SECRET.to_string(),
            GA_TRACKER_MEASUREMENT_ID.to_string(),
        ));
        let posthog_tracker = Box::new(posthog::Tracker::new(POSTHOG_API_SECRET.to_string()));
        let start_time = Utc::now();
        Self {
            collectors: vec![ga_tracker, posthog_tracker],
            is_tracking: can_track(),
            start_time,
        }
    }
}

impl Tracker {
    pub async fn init_ping(&'static self, duration: Duration) {
        if self.is_tracking {
            let mut interval = tokio::time::interval(duration);
            tokio::task::spawn(async move {
                loop {
                    interval.tick().await;
                    let _ = self.dispatch(EventKind::Ping).await;
                }
            });
        }
    }

    pub async fn dispatch(&'static self, event_kind: EventKind) -> Result<()> {
        if self.is_tracking {
            // Create a new event
            let event = Event {
                event_name: event_kind.as_str().to_string(),
                start_time: self.start_time,
                cores: cores(),
                client_id: client_id(),
                os_name: os_name(),
                up_time: self.up_time(event_kind),
                args: args(),
                path: path(),
                cwd: cwd(),
                user: user(),
                version: version(),
            };

            // Dispatch the event to all collectors
            for collector in &self.collectors {
                collector.collect(event.clone()).await?;
            }

            tracing::debug!("Event dispatched: {:?}", event);
        }

        Ok(())
    }

    fn up_time(&self, event_kind: EventKind) -> Option<String> {
        match event_kind {
            EventKind::Ping => Some(get_uptime(self.start_time)),
            _ => None,
        }
    }
}

// Generates a random client ID
fn client_id() -> String {
    let mut builder = IdBuilder::new(Encryption::SHA256);
    builder
        .add_component(HWIDComponent::SystemID)
        .add_component(HWIDComponent::CPUCores);
    builder
        .build(PARAPHRASE)
        .unwrap_or(DEFAULT_CLIENT_ID.to_string())
}

// Get the number of CPU cores
fn cores() -> usize {
    let sys = System::new_all();
    sys.physical_core_count().unwrap_or(0)
}

// Get the uptime in minutes
fn get_uptime(start_time: DateTime<Utc>) -> String {
    let current_time = Utc::now();
    format!(
        "{} minutes",
        current_time.signed_duration_since(start_time).num_minutes()
    )
}

fn version() -> String {
    tailcall_version::VERSION.as_str().to_string()
}

fn user() -> String {
    whoami::username()
}

fn cwd() -> Option<String> {
    std::env::current_dir()
        .ok()
        .and_then(|path| path.to_str().map(|s| s.to_string()))
}

fn path() -> Option<String> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.to_str().map(|s| s.to_string()))
}

fn args() -> Vec<String> {
    std::env::args().skip(1).collect()
}

fn os_name() -> String {
    System::long_os_version().unwrap_or("Unknown".to_string())
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use super::*;

    lazy_static! {
        static ref TRACKER: Tracker = Tracker::default();
    }

    #[tokio::test]
    async fn test_tracker() {
        if let Err(e) = TRACKER.dispatch(EventKind::Run).await {
            panic!("Tracker dispatch error: {:?}", e);
        }
    }
}
