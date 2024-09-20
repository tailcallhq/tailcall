use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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
