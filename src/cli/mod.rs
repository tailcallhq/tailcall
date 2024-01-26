pub mod cache;
mod command;
pub(crate) mod env;
mod error;
pub(crate) mod file;
mod fmt;
pub(crate) mod http;
pub mod javascript;
mod proto_path_resolver;
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

use crate::channel::{Command, Event};
use crate::cli::proto_path_resolver::NativeProtoPathResolver;
use crate::config::Upstream;
use crate::{blueprint, EnvIO, FileIO, HttpIO, ProtoPathResolver, ScriptIO};

// Provides access to env in native rust environment
pub fn init_env() -> Arc<dyn EnvIO> {
    Arc::new(env::EnvNative::init())
}

// Provides access to file system in native rust environment
pub fn init_file() -> Arc<dyn FileIO> {
    Arc::new(file::NativeFileIO::init())
}

pub fn init_hook_http(http: impl HttpIO, script: Option<blueprint::Script>) -> Arc<dyn HttpIO> {
    if let Some(script) = script {
        let script_io = javascript::Runtime::new(script);
        Arc::new(javascript::HttpFilter::new(http, script_io))
    } else {
        Arc::new(http)
    }
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
pub fn init_script(script: blueprint::Script) -> Arc<dyn ScriptIO<Event, Command>> {
    Arc::new(javascript::Runtime::new(script))
}

pub fn init_proto_resolver() -> Arc<dyn ProtoPathResolver> {
    Arc::new(NativeProtoPathResolver::new())
}
