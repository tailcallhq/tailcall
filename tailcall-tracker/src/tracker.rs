use chrono::Utc;
use tokio::time::Duration;

use super::Result;
use crate::check_tracking::check_tracking;
use crate::collect::collectors::Collectors;
use crate::collect::{ga, posthog};
use crate::EventKind;

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

pub struct Tracker {
    collectors: Collectors,
    is_tracking: bool,
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
            collectors: Collectors::new(start_time, vec![ga_tracker, posthog_tracker]),
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
                    let _ = self.collectors.dispatch(EventKind::Ping).await;
                }
            });
        }
    }

    pub async fn dispatch(&'static self, event: EventKind) -> Result<()> {
        if self.is_tracking {
            self.collectors.dispatch(event).await?;
        }
        Ok(())
    }
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
        if let Err(e) = TRACKER
            .dispatch(EventKind::Run {
                command: "test".to_string(),
                args: vec!["test_user".to_string(), "/test_path".to_string()],
            })
            .await
        {
            panic!("Tracker dispatch error: {:?}", e);
        }
    }
}
