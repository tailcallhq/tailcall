pub mod cache;
mod command;
mod error;
mod fmt;
pub mod server;
mod tc;

use std::hash::Hash;

pub use error::CLIError;
pub use tc::run;

use crate::config::Upstream;
use crate::HttpIO;

pub(crate) mod env;
pub(crate) mod file;
pub(crate) mod http;
use cache::NativeChronoCache;
pub use env::EnvNative;
pub use file::NativeFileIO;
pub use http::NativeHttp;

// Provides access to env in native rust environment
pub fn init_env() -> env::EnvNative {
  env::EnvNative::init()
}

// Provides access to file system in native rust environment
pub fn init_file() -> file::NativeFileIO {
  file::NativeFileIO::init()
}

// Provides access to http in native rust environment
pub fn init_http(upstream: &Upstream) -> http::NativeHttp {
  http::NativeHttp::init(upstream)
}

// Provides access to http in native rust environment
pub fn init_http2_only(upstream: &Upstream) -> http::NativeHttp {
  http::NativeHttp::init(&upstream.clone().http2_only(true))
}

pub fn init_chrono_cahe<K: Hash + Eq, V: Clone>() -> NativeChronoCache<K, V> {
  NativeChronoCache::new()
}
