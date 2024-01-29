pub mod cache;
mod command;
pub(crate) mod env;
mod error;
pub(crate) mod file;
mod fmt;
pub(crate) mod http;
#[cfg(feature = "js")]
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

// use crate::config::Upstream;
use crate::blueprint::Upstream;
use crate::{blueprint, EnvIO, FileIO, HttpIO};

// Provides access to env in native rust environment
pub fn init_env() -> Arc<dyn EnvIO> {
    Arc::new(env::EnvNative::init())
}

// Provides access to file system in native rust environment
pub fn init_file() -> Arc<dyn FileIO> {
    Arc::new(file::NativeFileIO::init())
}

pub fn init_hook_http(http: impl HttpIO, script: Option<blueprint::Script>) -> Arc<dyn HttpIO> {
    #[cfg(feature = "js")]
    if let Some(script) = script {
        return javascript::init_http(http, script);
    }

    #[cfg(not(feature = "js"))]
    log::warn!("JS capabilities are disabled in this build");
    let _ = script;

    Arc::new(http)
}

// Provides access to http in native rust environment
pub fn init_http(upstream: &Upstream, script: Option<blueprint::Script>) -> Arc<dyn HttpIO> {
    let http_io = http::NativeHttp::init(upstream);
    init_hook_http(http_io, script)
}

// Provides access to http in native rust environment
pub fn init_http2_only(upstream: &Upstream, script: Option<blueprint::Script>) -> Arc<dyn HttpIO> {
    let http_io = http::NativeHttp::init(&upstream.clone().http2_only(true));
    init_hook_http(http_io, script)
}

pub fn init_chrono_cache<K: Hash + Eq, V: Clone>() -> NativeChronoCache<K, V> {
    NativeChronoCache::new()
}
