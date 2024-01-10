use std::fmt::Display;

use anyhow::anyhow;

use crate::io::env::EnvIO;

pub struct EnvCloudflare {
  env: worker::Env,
}

impl EnvIO for EnvCloudflare {
  fn get(&self, key: &str) -> anyhow::Result<String> {
    let secret = self.env.secret(key).map_err(|e| anyhow!(e.to_string()))?;
    Ok(secret.to_string())
  }
}

impl EnvCloudflare {
  pub fn init(env: worker::Env) -> Self {
    Self { env }
  }
}

