use std::collections::HashMap;

use corex::EnvIO;

#[derive(Clone)]
pub struct EnvNative {
    vars: HashMap<String, String>,
}

impl EnvIO for EnvNative {
    fn get(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }
}

impl EnvNative {
    pub fn init() -> Self {
        Self { vars: std::env::vars().collect() }
    }
}
