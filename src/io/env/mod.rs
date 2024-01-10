#[cfg(feature = "default")]
mod native;
#[cfg(not(feature = "default"))]
mod cloudflare;

pub trait EnvIO: Send + Sync {
  fn get(&self, key: &str) -> anyhow::Result<String>;
}

#[cfg(feature = "default")]
pub fn init_env_native() -> impl EnvIO {
  native::EnvNative::init()
}

#[cfg(not(feature = "default"))]
pub fn init_env_cloudflare(env: worker::Env) -> impl EnvIO {
  cloudflare::EnvCloudflare::init(env)
}
