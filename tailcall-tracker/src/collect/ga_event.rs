use serde::{Deserialize, Serialize};

use crate::Event;

/// Event structure to be sent to GA
#[derive(Debug, Serialize, Deserialize)]
pub struct GaEvent {
    client_id: String,
    events: Vec<Event>,
}

impl GaEvent {
    pub fn new(event: Event) -> Self {
        Self { client_id: event.clone().client_id, events: vec![event] }
    }
}
