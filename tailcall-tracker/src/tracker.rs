use chrono::Utc;
use tokio::time::Duration;

use super::Result;
use crate::check_tracking::check_tracking;
use crate::collect::collectors::Collectors;
use crate::EventKind;

pub struct Tracker {
    collectors: Collectors,
    is_tracking: bool,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            collectors: Collectors::default(),
            is_tracking: check_tracking(),
        }
    }
}

impl Tracker {
    pub async fn init_ping(&'static self, duration: Duration) {
        if self.is_tracking {
            let mut interval = tokio::time::interval(duration);
            let start_time = Utc::now();
            tokio::task::spawn(async move {
                loop {
                    interval.tick().await;
                    let _ = self.collectors.dispatch(EventKind::Ping, start_time).await;
                }
            });
        }
    }

    pub async fn dispatch(&'static self, event: EventKind) -> Result<()> {
        if self.is_tracking {
            let start_time = Utc::now();
            self.collectors.dispatch(event, start_time).await?;
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
