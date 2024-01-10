#[cfg(not(feature = "default"))]
mod cloudflare;
#[cfg(feature = "default")]
mod native;

pub trait EnvIO {
  fn get(key: String) -> Option<String>;
}

#[cfg(feature = "default")]
pub fn init_env_native() -> impl EnvIO {
  todo!()
}

#[cfg(not(feature = "default"))]
pub fn init_env_cloudflare() -> impl EnvIO {
  todo!()
}
