use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub event_name: String,
    pub start_time: DateTime<Utc>,
    pub cores: usize,
    pub client_id: String,
    pub os_name: String,
    pub up_time: Option<String>,
    pub path: Option<String>,
    pub cwd: Option<String>,
    pub user: String,
    pub args: Vec<String>,
    pub version: String,
}

#[derive(Clone, IntoStaticStr)]
pub enum EventKind {
    Ping,
    Run,
}

impl EventKind {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}
