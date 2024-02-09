use std::collections::HashMap;

use crate::EnvIO;

#[derive(Clone)]
pub struct TestEnvIO {
    vars: HashMap<String, String>,
}

impl EnvIO for TestEnvIO {
    fn get(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }
}

impl TestEnvIO {
    pub fn init() -> Self {
        Self { vars: std::env::vars().collect() }
    }
}
