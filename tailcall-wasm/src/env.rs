use std::borrow::Cow;

use dashmap::DashMap;
use tailcall::core::EnvIO;

pub struct WasmEnv {
    env: DashMap<String, String>,
}

impl WasmEnv {
    pub fn init() -> Self {
        Self { env: DashMap::new() }
    }
    pub fn set(&self, key: String, value: String) {
        self.env.insert(key, value);
    }
}

impl EnvIO for WasmEnv {
    fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        self.env.get(key).map(|v| Cow::Owned(v.value().clone()))
    }
}
