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

pub(crate) mod update_checker;
pub use env::EnvNative;
pub use error::CLIError;
pub use file::NativeFileIO;
pub use http::NativeHttp;
pub use tc::run;

use crate::blueprint::Upstream;
use crate::cache::InMemoryCache;
use crate::runtime::TargetRuntime;
use crate::{blueprint, EnvIO, FileIO, HttpIO};

// Provides access to env in native rust environment
fn init_env() -> Arc<dyn EnvIO> {
    Arc::new(env::EnvNative::init())
}

// Provides access to file system in native rust environment
fn init_file() -> Arc<dyn FileIO> {
    Arc::new(file::NativeFileIO::init())
}

fn init_hook_http(http: impl HttpIO, script: Option<blueprint::Script>) -> Arc<dyn HttpIO> {
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
fn init_http(upstream: &Upstream, script: Option<blueprint::Script>) -> Arc<dyn HttpIO> {
    let http_io = http::NativeHttp::init(upstream);
    init_hook_http(http_io, script)
}

// Provides access to http in native rust environment
fn init_http2_only(upstream: &Upstream, script: Option<blueprint::Script>) -> Arc<dyn HttpIO> {
    let http_io = http::NativeHttp::init(&upstream.clone().http2_only(true));
    init_hook_http(http_io, script)
}

fn init_in_memory_cache<K: Hash + Eq, V: Clone>() -> InMemoryCache<K, V> {
    InMemoryCache::new()
}

pub fn init_runtime(upstream: &Upstream, script: Option<blueprint::Script>) -> TargetRuntime {
    TargetRuntime {
        http: init_http(upstream, script.clone()),
        http2_only: init_http2_only(upstream, script),
        env: init_env(),
        file: init_file(),
        cache: Arc::new(init_in_memory_cache()),
    }
}
