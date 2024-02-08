use std::collections::HashMap;

pub struct DataStore {
    store: HashMap<u64, serde_json::Value>,
}
