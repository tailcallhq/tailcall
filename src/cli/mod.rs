pub mod cache;
mod command;
pub(crate) mod env;
mod error;
pub(crate) mod file;
mod fmt;
pub(crate) mod http;
pub mod javascript;
pub mod server;
mod tc;
use std::hash::Hash;
use std::sync::Arc;

use cache::NativeChronoCache;
pub use env::EnvNative;
pub use error::CLIError;
pub use file::NativeFileIO;
pub use http::NativeHttp;
pub use tc::run;

#[cfg(feature = "js")]
use crate::channel::{Command, Event};
use crate::config::Upstream;
#[cfg(feature = "js")]
use crate::{blueprint, ScriptIO};
use crate::{EnvIO, FileIO, HttpIO};

// Provides access to env in native rust environment
pub fn init_env() -> Arc<dyn EnvIO> {
  Arc::new(env::EnvNative::init())
}

// Provides access to file system in native rust environment
pub fn init_file() -> Arc<dyn FileIO> {
  Arc::new(file::NativeFileIO::init())
}

pub fn init_hook_http(http: impl HttpIO, #[cfg(feature = "js")] script: Option<blueprint::Script>) -> Arc<dyn HttpIO> {
  #[cfg(feature = "js")]
  if let Some(script) = script {
    let script_io = javascript::Runtime::new(script);
    Arc::new(javascript::HttpFilter::new(http, script_io))
  } else {
    Arc::new(http)
  }
  #[cfg(not(feature = "js"))]
  Arc::new(http)
}

// Provides access to http in native rust environment
pub fn init_http(upstream: &Upstream, #[cfg(feature = "js")] script: Option<blueprint::Script>) -> Arc<dyn HttpIO> {
  let http_io = http::NativeHttp::init(upstream);
  init_hook_http(
    http_io,
    #[cfg(feature = "js")]
    script,
  )
}

// Provides access to http in native rust environment
pub fn init_http2_only(
  upstream: &Upstream,
  #[cfg(feature = "js")] script: Option<blueprint::Script>,
) -> Arc<dyn HttpIO> {
  let http_io = http::NativeHttp::init(&upstream.clone().http2_only(true));
  init_hook_http(
    http_io,
    #[cfg(feature = "js")]
    script,
  )
}

pub fn init_chrono_cache<K: Hash + Eq, V: Clone>() -> NativeChronoCache<K, V> {
  NativeChronoCache::new()
}
#[cfg(feature = "js")]
pub fn init_script(script: blueprint::Script) -> Arc<dyn ScriptIO<Event, Command>> {
  Arc::new(javascript::Runtime::new(script))
}
