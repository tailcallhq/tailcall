mod env;
mod file;
mod http;

use std::hash::Hash;
use std::sync::Arc;

pub use http::NativeHttp;

use crate::core::blueprint::Blueprint;
use crate::core::cache::InMemoryCache;
use crate::core::runtime::TargetRuntime;
use crate::core::worker::{Command, Event};
use crate::core::{blueprint, EnvIO, FileIO, HttpIO, WorkerIO};

// Provides access to env in native rust environment
fn init_env() -> Arc<dyn EnvIO> {
    Arc::new(env::EnvNative::init())
}

// Provides access to file system in native rust environment
fn init_file() -> Arc<dyn FileIO> {
    Arc::new(file::NativeFileIO::init())
}

fn init_hook_http(http: Arc<impl HttpIO>, script: Option<blueprint::Script>) -> Arc<dyn HttpIO> {
    #[cfg(feature = "js")]
    if let Some(script) = script {
        return super::javascript::init_http(http, script);
    }

    let _ = script;

    http
}

fn init_http_worker_io(
    _script: Option<blueprint::Script>,
) -> Option<Arc<dyn WorkerIO<Event, Command>>> {
    #[cfg(feature = "js")]
    return Some(super::javascript::init_worker_io(_script?));
    #[cfg(not(feature = "js"))]
    None
}

fn init_resolver_worker_io(
    _script: Option<blueprint::Script>,
) -> Option<Arc<dyn WorkerIO<async_graphql::Value, async_graphql::Value>>> {
    #[cfg(feature = "js")]
    return Some(super::javascript::init_worker_io(_script?));
    #[cfg(not(feature = "js"))]
    None
}

// Provides access to http in native rust environment
fn init_http(blueprint: &Blueprint) -> Arc<dyn HttpIO> {
    let http_io = http::NativeHttp::init(&blueprint.upstream, &blueprint.telemetry);
    init_hook_http(Arc::new(http_io), blueprint.server.script.clone())
}

// Provides access to http in native rust environment
fn init_http2_only(blueprint: &Blueprint) -> Arc<dyn HttpIO> {
    let http_io = http::NativeHttp::init(
        &blueprint.upstream.clone().http2_only(true),
        &blueprint.telemetry,
    );
    init_hook_http(Arc::new(http_io), blueprint.server.script.clone())
}

fn init_in_memory_cache<K: Hash + Eq, V: Clone>() -> InMemoryCache<K, V> {
    InMemoryCache::new()
}

pub fn init(blueprint: &Blueprint) -> TargetRuntime {
    #[cfg(not(feature = "js"))]
    tracing::warn!("JS capabilities are disabled in this build");

    TargetRuntime {
        http: init_http(blueprint),
        http2_only: init_http2_only(blueprint),
        env: init_env(),
        file: init_file(),
        cache: Arc::new(init_in_memory_cache()),
        extensions: Arc::new(vec![]),
        http_worker: init_http_worker_io(blueprint.server.script.clone()),
        worker: init_resolver_worker_io(blueprint.server.script.clone()),
    }
}
