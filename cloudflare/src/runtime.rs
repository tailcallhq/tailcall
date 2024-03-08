use std::rc::Rc;
use std::sync::Arc;

use anyhow::anyhow;
use async_graphql_value::ConstValue;
use tailcall::runtime::TargetRuntime;
use tailcall::{EnvIO, FileIO, HttpIO};

use crate::{cache, env, file, http};

fn init_env(env: Rc<worker::Env>) -> Arc<dyn EnvIO> {
    Arc::new(env::CloudflareEnv::init(env))
}

fn init_file(env: Rc<worker::Env>, bucket_id: String) -> anyhow::Result<Arc<dyn FileIO>> {
    Ok(Arc::new(file::CloudflareFileIO::init(env, bucket_id)?))
}

fn init_http() -> Arc<dyn HttpIO> {
    Arc::new(http::CloudflareHttp::init())
}

fn init_cache(env: Rc<worker::Env>) -> Arc<dyn tailcall::Cache<Key = u64, Value = ConstValue>> {
    Arc::new(cache::CloudflareChronoCache::init(env))
}

pub fn init(env: Rc<worker::Env>) -> anyhow::Result<TargetRuntime> {
    let http = init_http();
    let env_io = init_env(env.clone());
    let bucket_id = env_io
        .get("BUCKET")
        .ok_or(anyhow!("BUCKET var is not set"))?;

    Ok(TargetRuntime {
        http: http.clone(),
        http2_only: http.clone(),
        env: init_env(env.clone()),
        file: init_file(env.clone(), bucket_id)?,
        cache: init_cache(env),
        extensions: vec![],
    })
}
