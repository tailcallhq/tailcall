use chrono::{DateTime, Utc};
use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
use sysinfo::System;

use super::super::Result;
use super::ga::GaTracker;
use super::posthog::PostHogTracker;
use crate::tracker::{Event, EventKind};

const PARAPHRASE: &str = "tc_key";
const DEFAULT_CLIENT_ID: &str = "<anonymous>";

#[async_trait::async_trait]
pub trait EventCollector: Send + Sync {
    async fn dispatch(&self, event: Event) -> Result<()>;
}

pub struct Collectors {
    collectors: Vec<Box<dyn EventCollector>>,
}

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
    pub async fn dispatch(
        &'static self,
        event_kind: EventKind,
        start_time: DateTime<Utc>,
    ) -> Result<()> {
        let event = Event {
            event_name: event_kind.to_string(),
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
    fn get_client_id() -> String {
        let mut builder = IdBuilder::new(Encryption::SHA256);
        builder
            .add_component(HWIDComponent::SystemID)
            .add_component(HWIDComponent::CPUCores);
        builder
            .build(PARAPHRASE)
            .unwrap_or(DEFAULT_CLIENT_ID.to_string())
    }
    fn get_cpu_cores() -> usize {
        let sys = System::new_all();
        sys.physical_core_count().unwrap_or(2)
    }
    fn get_os_name() -> String {
        System::long_os_version().unwrap_or("Unknown".to_string())
    }

    fn get_uptime(start_time: DateTime<Utc>) -> String {
        let current_time = Utc::now();
        format!(
            "{} minutes",
            current_time.signed_duration_since(start_time).num_minutes()
        )
    }
}
