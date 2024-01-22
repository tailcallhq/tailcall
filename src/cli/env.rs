use std::{borrow::Cow, collections::HashMap};

use crate::EnvIO;

#[derive(Clone)]
pub struct EnvNative {
  vars: HashMap<String, String>,
}

impl EnvIO for EnvNative {
  fn get(&self, key: &str) -> Option<Cow<'_, str>> {
    self.vars.get(key).map(|s| Cow::Borrowed(s.as_str()))
  }
}

impl EnvNative {
  pub fn init() -> Self {
    Self { vars: std::env::vars().collect() }
  }
}
