extern crate core;

use std::borrow::Cow;
use std::collections::HashMap;

use tailcall::core::EnvIO;

#[derive(Clone)]
pub struct Env {
    vars: HashMap<String, String>,
}

impl EnvIO for Env {
    fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        self.vars.get(key).map(Cow::from)
    }
}

impl Env {
    pub fn init(vars: Option<HashMap<String, String>>) -> Self {
        Self { vars: vars.unwrap_or_default() }
    }
}
