use chrono::{DateTime, Utc};
use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
use sysinfo::System;

use super::super::Result;
use super::ga::GaTracker;
use super::posthog::PostHogTracker;
use crate::{Event, EventKind};

const PARAPHRASE: &str = "tc_key";
const DEFAULT_CLIENT_ID: &str = "<anonymous>";

///
/// Dispatches events to multiple collectors.
///
#[async_trait::async_trait]
pub trait EventCollector: Send + Sync {
    async fn dispatch(&self, event: Event) -> Result<()>;
}

pub struct Collectors {
    collectors: Vec<Box<dyn EventCollector>>,
}

// FIXME: implement a `new` method for `Collectors` to initialize the collectors
impl Default for Collectors {
    fn default() -> Self {
        let ga_tracker = GaTracker::default();
        let posthog_tracker = PostHogTracker::default();
        Self {
            collectors: vec![Box::new(ga_tracker), Box::new(posthog_tracker)],
        }
    }
}

impl Collectors {
    /// Dispatches an event to all collectors.
    pub async fn dispatch(
        &'static self,
        event_kind: EventKind,
        // FIXME: can be initialized within Self
        start_time: DateTime<Utc>,
    ) -> Result<()> {
        let event = Event {
            event_name: event_kind.name(),
            start_time,
            cores: Self::get_cpu_cores(),
            client_id: Self::get_client_id(),
            os_name: Self::get_os_name(),
            up_time: match event_kind {
                EventKind::Ping => Some(Self::get_uptime(start_time)),
                _ => None,
            },
            args: match event_kind {
                EventKind::Run { args, .. } => Some(args),
                _ => None,
            },
            path: None,
            user: None,
        };
        for collector in &self.collectors {
            collector.dispatch(event.clone()).await?;
        }
        Ok(())
    }

    // Generates a random client ID
    fn get_client_id() -> String {
        let mut builder = IdBuilder::new(Encryption::SHA256);
        builder
            .add_component(HWIDComponent::SystemID)
            .add_component(HWIDComponent::CPUCores);
        builder
            .build(PARAPHRASE)
            .unwrap_or(DEFAULT_CLIENT_ID.to_string())
    }

    // Get the number of CPU cores
    fn get_cpu_cores() -> usize {
        let sys = System::new_all();
        sys.physical_core_count().unwrap_or(2)
    }

    // Get the OS name
    fn get_os_name() -> String {
        System::long_os_version().unwrap_or("Unknown".to_string())
    }

    // Get the uptime in minutes
    fn get_uptime(start_time: DateTime<Utc>) -> String {
        let current_time = Utc::now();
        format!(
            "{} minutes",
            current_time.signed_duration_since(start_time).num_minutes()
        )
    }
}
