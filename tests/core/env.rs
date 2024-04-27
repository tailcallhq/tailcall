extern crate core;

use std::borrow::Cow;
use std::collections::HashMap;

use tailcall::EnvIO;

#[derive(Clone)]
pub struct TestEnvIO {
    vars: HashMap<String, String>,
}

impl EnvIO for TestEnvIO {
    fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        self.vars.get(key).map(Cow::from)
    }
}

impl TestEnvIO {
    pub fn init(vars: Option<HashMap<String, String>>) -> Self {
        Self { vars: vars.unwrap_or_default() }
    }
}
