use std::collections::HashMap;

use anyhow::anyhow;

#[cfg(not(feature = "default"))]
mod cloudflare;
#[cfg(feature = "default")]
mod native;

pub trait EnvIO: Send + Sync {
  fn get(&self, key: &str) -> anyhow::Result<String>;
}

#[cfg(feature = "default")]
pub fn init_env_native() -> impl EnvIO {
  native::EnvNative::init()
}

pub fn init_env_test(map: HashMap<String, String>) -> impl EnvIO {
  EnvTest::init(map)
}

#[cfg(not(feature = "default"))]
pub fn init_env_cloudflare(env: worker::Env) -> impl EnvIO {
  cloudflare::EnvCloudflare::init(env)
}

struct EnvTest {
  env: HashMap<String, String>,
}

impl EnvIO for EnvTest {
  fn get(&self, key: &str) -> anyhow::Result<String> {
    self.env.get(key).ok_or(anyhow!("Key not found")).cloned()
  }
}
impl EnvTest {
  fn init(map: HashMap<String, String>) -> Self {
    Self { env: map }
  }
}
