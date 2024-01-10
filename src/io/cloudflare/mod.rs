use crate::io::{EnvIO, FileIO, HttpIO};

mod env;
mod file;
mod http;

pub fn init_env(env: worker::Env) -> impl EnvIO {
  env::EnvCloudflare::init(env)
}

pub fn init_file() -> impl FileIO {
  file::CloudflareFileIO::init()
}

pub fn init_http() -> impl HttpIO + Default + Clone {
  http::HttpCloudflare::init()
}
