pub mod cache;
mod command;
pub(crate) mod env;
mod error;
pub(crate) mod file;
mod fmt;
pub(crate) mod http;
mod http_hook;
pub mod script;
pub mod server;
mod tc;
use std::hash::Hash;

use cache::NativeChronoCache;
pub use env::EnvNative;
pub use error::CLIError;
pub use file::NativeFileIO;
pub use http::NativeHttp;
pub use tc::run;

use crate::config::Upstream;
use crate::HttpIO;

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

pub fn init_chrono_cache<K: Hash + Eq, V: Clone>() -> NativeChronoCache<K, V> {
  NativeChronoCache::new()
}
