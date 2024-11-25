use std::ops::Deref;

use chrono::{DateTime, Utc};
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub event_name: Name,
    pub start_time: DateTime<Utc>,
    pub cores: usize,
    pub client_id: String,
    pub os_name: String,
    pub up_time: i64,
    pub path: Option<String>,
    pub cwd: Option<String>,
    pub user: String,
    pub args: Vec<String>,
    pub version: String,
    pub email: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Name(String);
impl From<String> for Name {
    fn from(name: String) -> Self {
        Self(name.to_case(Case::Snake))
    }
}
impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Name> for String {
    fn from(val: Name) -> Self {
        val.0
    }
}

#[derive(Debug, Clone)]
pub enum EventKind {
    Ping,
    Command(String),
}

impl EventKind {
    pub fn name(&self) -> Name {
        match self {
            Self::Ping => Name::from("ping".to_string()),
            Self::Command(name) => Name::from(name.to_string()),
        }
    }
}
