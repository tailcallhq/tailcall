use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

use super::Result;
use crate::{check_tracking::check_tracking, collect::collectors::Collectors};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub event_name: String,
    pub start_time: DateTime<Utc>,
    pub cores: usize,
    pub client_id: String,
    pub os_name: String,
    pub up_time: Option<String>,
    pub path: Option<String>,
    pub user: Option<String>,
    pub args: Option<Vec<String>>,
}

#[derive(Clone)]
pub enum EventKind {
    Ping,
    Run { command: String, args: Vec<String> },
}

impl Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventKind::Ping => write!(f, "ping"),
            EventKind::Run { command, .. } => write!(f, "{}", command),
        }
    }
}

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
    use super::*;
    use lazy_static::lazy_static;

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
