use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::helpers::{get_client_id, get_cpu_cores, get_os_name, get_uptime};

#[derive(Debug, Serialize, Deserialize)]
struct Params {
    cpu_cores: String,
    os_name: String,
    start_time: String,
    uptime: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GaEventValue {
    name: String,
    params: Params,
}

impl GaEventValue {
    fn new(name: &str, start_time: DateTime<Utc>) -> Self {
        let cores = get_cpu_cores();
        let os_name = get_os_name();
        let mut uptime = None;
        if name == "ping" {
            uptime = Some(get_uptime(start_time));
        }
        GaEventValue {
            name: name.to_string(),
            params: Params {
                cpu_cores: cores,
                os_name,
                start_time: start_time.to_string(),
                uptime,
            },
        }
    }
}

/// Event structure to be sent to GA
#[derive(Debug, Serialize, Deserialize)]
pub struct GaEvent {
    client_id: String,
    events: Vec<GaEventValue>,
}

impl GaEvent {
    pub fn new(name: &str, start_time: DateTime<Utc>) -> Self {
        let id = get_client_id();

        Self {
            client_id: id,
            events: vec![GaEventValue::new(name, start_time)],
        }
    }
}
