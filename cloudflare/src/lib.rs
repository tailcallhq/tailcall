use std::rc::Rc;

use anyhow::anyhow;
use tailcall::io::{EnvIO, FileIO, HttpIO};

mod env;
mod file;
mod handle;
mod http;
mod r2_address;

pub fn init_env(env: Rc<worker::Env>) -> impl EnvIO {
  env::EnvCloudflare::init(env)
}

pub fn init_file(env: Rc<worker::Env>) -> impl FileIO {
  file::CloudflareFileIO::init(env)
}

pub fn init_http() -> impl HttpIO + Default + Clone {
  http::HttpCloudflare::init()
}

#[worker::event(fetch)]
async fn main(req: worker::Request, env: worker::Env, context: worker::Context) -> anyhow::Result<worker::Response> {
  Ok(handle::execute(req, env, context).await?)
}

fn to_anyhow<T: std::fmt::Display>(e: T) -> anyhow::Error {
  anyhow!("{}", e)
}
