use std::sync::Arc;

use anyhow::anyhow;
use tailcall::io::{EnvIO, FileIO, HttpIO};

mod env;
mod file;
mod handle;
mod http;
mod r2_address;

pub fn init_env(env: Arc<worker::Env>) -> impl EnvIO {
  env::EnvCloudflare::init(env)
}

pub fn init_file(env: Arc<worker::Env>) -> impl FileIO {
  file::CloudflareFileIO::init(env)
}

pub fn init_http() -> impl HttpIO + Default + Clone {
  http::HttpCloudflare::init()
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
}

fn to_anyhow<T: std::fmt::Display>(e: T) -> anyhow::Error {
  anyhow!("{}", e)
}
