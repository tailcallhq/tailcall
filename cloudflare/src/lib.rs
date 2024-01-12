use std::sync::Arc;

use anyhow::anyhow;
use tailcall::io::{EnvIO, FileIO, HttpIO};

mod env;
mod file;
mod handle;
mod http;
mod r2_address;

fn init_env(env: Arc<worker::Env>) -> impl EnvIO {
  env::EnvCloudflare::init(env)
}

fn init_file(env: Arc<worker::Env>) -> impl FileIO {
  file::CloudflareFileIO::init(env)
}

fn init_http() -> impl HttpIO + Default + Clone {
  http::HttpCloudflare::init()
}

#[worker::event(fetch)]
async fn main(req: worker::Request, env: worker::Env, context: worker::Context) -> anyhow::Result<worker::Response> {
  Ok(handle::execute(req, env, context).await?)
}

fn to_anyhow<T: std::fmt::Display>(e: T) -> anyhow::Error {
  anyhow!("{}", e)
}
