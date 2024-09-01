use super::Result;
use crate::{
    check_tracking::check_tracking,
    collect::{ga::GATracker, posthog::PostHogTracker},
};
use tokio::time::Duration;

#[async_trait::async_trait]
pub trait EventCollector: Send + Sync {
    async fn dispatch(&self, event_name: &str) -> Result<()>;
}

pub struct Tracker {
    collectors: Vec<Box<dyn EventCollector>>,
    is_tracking: bool,
}

impl Default for Tracker {
    fn default() -> Self {
        let ga_tracker = GATracker::default();
        let posthog_tracker = PostHogTracker::default();
        Self {
            collectors: vec![Box::new(ga_tracker), Box::new(posthog_tracker)],
            is_tracking: check_tracking(),
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
                    let _ = self.dispatch("ping").await;
                }
            });
        }
    }

    pub async fn dispatch(&'static self, name: &str) -> Result<()> {
        if self.is_tracking {
            for collector in &self.collectors {
                collector.dispatch(name).await?;
            }
        }
        Ok(())
    }
}
