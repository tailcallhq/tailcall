use std::collections::HashMap;

use tailcall::io::EnvIO;

#[derive(Clone)]
pub struct EnvCloudflare {
  env: HashMap<String, String>,
}

impl EnvIO for EnvCloudflare {
  fn get(&self, key: &str) -> Option<String> {
    self.env.get(key).cloned()
  }
}

impl EnvCloudflare {
  pub fn init(env: HashMap<String, String>) -> Self {
    Self { env }
  }
}
