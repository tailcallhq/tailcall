use std::panic;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::anyhow;
use async_graphql_value::ConstValue;
use tailcall::{EnvIO, FileIO, HttpIO, ProtoPathResolver};
use crate::proto_resolver::CloudflareProtoPathResolver;

mod cache;
mod env;
mod file;
pub mod handle;
mod http;
mod proto_resolver;

pub fn init_env(env: Rc<worker::Env>) -> Arc<dyn EnvIO> {
    Arc::new(env::CloudflareEnv::init(env))
}

pub fn init_file(env: Rc<worker::Env>, bucket_id: String) -> anyhow::Result<Arc<dyn FileIO>> {
    Ok(Arc::new(file::CloudflareFileIO::init(env, bucket_id)?))
}

pub fn init_proto_resolver() -> Arc<dyn ProtoPathResolver> {
    Arc::new(CloudflareProtoPathResolver::init())
}

pub fn init_http() -> Arc<dyn HttpIO> {
    Arc::new(http::CloudflareHttp::init())
}

pub fn init_cache(env: Rc<worker::Env>) -> Arc<dyn tailcall::Cache<Key = u64, Value = ConstValue>> {
    Arc::new(cache::CloudflareChronoCache::init(env))
}

#[worker::event(fetch)]
async fn fetch(
    req: worker::Request,
    env: worker::Env,
    _: worker::Context,
) -> anyhow::Result<worker::Response> {
    let result = handle::fetch(req, env).await;

    match result {
        Ok(response) => Ok(response),
        Err(message) => {
            log::error!("ServerError: {}", message.to_string());
            worker::Response::error(message.to_string(), 500).map_err(to_anyhow)
        }
    }
}

#[worker::event(start)]
fn start() {
    // Initialize Logger
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

fn to_anyhow<T: std::fmt::Display>(e: T) -> anyhow::Error {
    anyhow!("{}", e)
}
