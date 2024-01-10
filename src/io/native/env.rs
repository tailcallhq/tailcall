use std::collections::HashMap;

use anyhow::anyhow;

use crate::io::EnvIO;

pub struct EnvNative {
  vars: HashMap<String, String>,
}

impl EnvIO for EnvNative {
  fn get(&self, key: &str) -> anyhow::Result<String> {
    self.vars.get(key).cloned().ok_or(anyhow!("Key not found"))
  }
}

impl EnvNative {
  pub fn init() -> Self {
    Self { vars: std::env::vars().collect() }
  }
}
