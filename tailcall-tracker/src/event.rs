use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

impl EventKind {
    pub fn name(&self) -> String {
        match self {
            EventKind::Ping => "ping".to_string(),
            EventKind::Run { command, .. } => format!("{}", command.to_lowercase()),
        }
    }
}
