use std::sync::Arc;
use tailcall::io::EnvIO;
use worker::Env;

pub struct EnvCloudflare {
  env: Arc<Env>,
}

unsafe impl Send for EnvCloudflare {}
unsafe impl Sync for EnvCloudflare {}

impl EnvIO for EnvCloudflare {
  fn get(&self, key: &str) -> Option<String> {
    self.env.var(key).ok().map(|s| s.to_string())
  }
}

impl EnvCloudflare {
  pub fn init(env: Arc<Env>) -> Self {
    Self { env }
  }
}
