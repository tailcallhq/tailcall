use std::rc::Rc;

use anyhow::anyhow;

mod env;
mod file;
mod handle;
mod http;

pub fn init_env(env: Rc<worker::Env>) -> env::CloudflareEnv {
  env::CloudflareEnv::init(env)
}

pub fn init_file_r2(env: Rc<worker::Env>, id: String) -> file::CloudflareR2FileIO {
  file::CloudflareR2FileIO::init(env, id)
}

pub fn init_file_static(env: Rc<worker::Env>) -> file::CloudflareStaticFileIO {
  file::CloudflareStaticFileIO::init(env)
}

pub fn init_http() -> http::CloudflareHttp {
  http::CloudflareHttp::init()
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
