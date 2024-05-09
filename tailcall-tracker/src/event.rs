use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
use serde::{Deserialize, Serialize};
use sysinfo::System;

const PARAPHRASE: &str = "tc_key";
const DEFAULT_CLIENT_ID: &str = "<anonymous>";

#[derive(Debug, Serialize, Deserialize)]
struct Params {
    cpu_cores: String,
    os_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventValue {
    name: String,
    params: Params,
}

impl EventValue {
    fn new(name: &str) -> EventValue {
        let sys = System::new_all();
        let cores = sys.physical_core_count().unwrap_or(2).to_string();
        let os_name = System::long_os_version().unwrap_or("Unknown".to_string());
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
        let mut builder = IdBuilder::new(Encryption::SHA256);
        builder
            .add_component(HWIDComponent::SystemID)
            .add_component(HWIDComponent::CPUCores);

        let id = builder
            .build(PARAPHRASE)
            .unwrap_or(DEFAULT_CLIENT_ID.to_string());

        Self { client_id: id, events: vec![EventValue::new(name)] }
    }
}
