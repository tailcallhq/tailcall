extern crate core;

use std::borrow::Cow;

use tailcall::EnvIO;
use tailcall_hasher::TailcallHashMap;

#[derive(Clone)]
pub struct Env {
    vars: TailcallHashMap<String, String>,
}

impl EnvIO for Env {
    fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        self.vars.get(key).map(Cow::from)
    }
}

impl Env {
    pub fn init(vars: Option<TailcallHashMap<String, String>>) -> Self {
        Self { vars: vars.unwrap_or_default() }
    }
}
