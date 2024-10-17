use std::sync::Arc;

use async_graphql_value::ConstValue;
use tailcall::core::cache::InMemoryCache;
use tailcall::core::ir::model::IoId;
use tailcall::core::runtime::TargetRuntime;
use tailcall::core::{EnvIO, FileIO, HttpIO};

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

fn init_cache() -> Arc<InMemoryCache<IoId, ConstValue>> {
    Arc::new(InMemoryCache::default())
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
        cmd_worker: None,
        worker: None,
    }
}
