use std::sync::Arc;

use tailcall::{EnvIO, FileIO, HttpIO, TargetRuntime};

use crate::cache::WasmCache;
use crate::env::WasmEnv;
use crate::file::WasmFile;
use crate::http::WasmHttp;

fn init_http() -> Arc<dyn HttpIO> {
    Arc::new(WasmHttp::init())
}

fn init_file() -> Arc<dyn FileIO> {
    Arc::new(WasmFile::init())
}

fn init_env() -> Arc<dyn EnvIO> {
    Arc::new(WasmEnv::init())
}

fn init_cache() -> Arc<WasmCache> {
    Arc::new(WasmCache::init())
}

pub fn init_rt() -> TargetRuntime {
    let http = init_http();
    let http2_only = init_http();
    let file = init_file();
    let env = init_env();
    let cache = init_cache();
    TargetRuntime {
        http,
        http2_only,
        env,
        file,
        cache,
        extensions: Arc::new(vec![]),
    }
}
