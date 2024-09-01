use chrono::{DateTime, Utc};
use tokio::time::Duration;

use super::Result;
use crate::check_tracking::check_tracking;
use crate::collect::ga::GaTracker;
use crate::collect::posthog::PostHogTracker;

#[async_trait::async_trait]
pub trait EventCollector: Send + Sync {
    async fn dispatch(&self, event_name: &str, start_time: DateTime<Utc>) -> Result<()>;
}

pub struct Tracker {
    collectors: Vec<Box<dyn EventCollector>>,
    is_tracking: bool,
}

impl Default for Tracker {
    fn default() -> Self {
        let ga_tracker = GaTracker::default();
        let posthog_tracker = PostHogTracker::default();
        Self {
            collectors: vec![Box::new(ga_tracker), Box::new(posthog_tracker)],
            is_tracking: check_tracking(),
        }
    }
}

impl Tracker {
    pub async fn init_ping(&'static self, duration: Duration, start_time: DateTime<Utc>) {
        if self.is_tracking {
            let mut interval = tokio::time::interval(duration);
            tokio::task::spawn(async move {
                loop {
                    interval.tick().await;
                    let _ = self.dispatch("ping", start_time).await;
                }
            });
        }
    }

    pub async fn dispatch(&'static self, name: &str, start_time: DateTime<Utc>) -> Result<()> {
        if self.is_tracking {
            for collector in &self.collectors {
                collector.dispatch(name, start_time).await?;
            }
        }
        Ok(())
    }
}
