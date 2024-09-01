use crate::helpers::{get_client_id, get_cpu_cores, get_os_name};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Params {
    cpu_cores: String,
    os_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventValue {
    name: String,
    params: Params,
}

impl EventValue {
    fn new(name: &str) -> EventValue {
        let cores = get_cpu_cores();
        let os_name = get_os_name();
        EventValue {
            name: name.to_string(),
            params: Params { cpu_cores: cores, os_name },
        }
    }
}

/// Event structure to be sent to GA
#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    client_id: String,
    events: Vec<EventValue>,
}

impl Event {
    pub fn new(name: &str) -> Self {
        let id = get_client_id();

        Self { client_id: id, events: vec![EventValue::new(name)] }
    }
}
