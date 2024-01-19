use std::rc::Rc;

use anyhow::anyhow;

use std::panic;

mod cache;
mod env;
mod file;
mod handle;
mod http;

pub fn init_env(env: Rc<worker::Env>) -> env::CloudflareEnv {
  env::CloudflareEnv::init(env)
}

pub fn init_file(env: Rc<worker::Env>, bucket_id: String) -> anyhow::Result<file::CloudflareFileIO> {
  file::CloudflareFileIO::init(env, bucket_id)
}

pub fn init_http() -> http::CloudflareHttp {
  http::CloudflareHttp::init()
}

pub fn init_cache(env: Rc<worker::Env>) -> cache::CloudflareChronoCache {
  cache::CloudflareChronoCache::init(env)
}

#[worker::event(fetch)]
async fn fetch(req: worker::Request, env: worker::Env, context: worker::Context) -> anyhow::Result<worker::Response> {
  let result = handle::fetch(req, env, context).await;

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
