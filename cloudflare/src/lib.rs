use std::rc::Rc;

use anyhow::anyhow;

mod env;
mod file;
mod handle;
mod http;
mod r2_address;

pub fn init_env(env: Rc<worker::Env>) -> env::EnvCloudflare {
  env::EnvCloudflare::init(env)
}

pub fn init_file(env: Rc<worker::Env>) -> file::CloudflareFileIO {
  file::CloudflareFileIO::init(env)
}

pub fn init_http() -> http::HttpCloudflare {
  http::HttpCloudflare::init()
}

#[worker::event(fetch)]
async fn fetch(req: worker::Request, env: worker::Env, context: worker::Context) -> anyhow::Result<worker::Response> {
  let result = handle::fetch(req, env, context).await;

  match result {
    Ok(response) => Ok(response),
    Err(message) => {
      tracing::error!("ServerError: {}", message.to_string());
      worker::Response::error(message.to_string(), 500).map_err(to_anyhow)
    }
  }
}

#[worker::event(start)]
fn start() {
  // Initialize Logger
  let config = tracing_wasm::WASMLayerConfigBuilder::new()
    .set_max_level(tracing::Level::INFO)
    .build();

  tracing_wasm::set_as_global_default_with_config(config)
}

fn to_anyhow<T: std::fmt::Display>(e: T) -> anyhow::Error {
  anyhow!("{}", e)
}
