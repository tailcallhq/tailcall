use crate::config::Upstream;
use crate::http::HttpClientOptions;
use crate::io::{EnvIO, FileIO, HttpIO};

pub(crate) mod env;
pub(crate) mod file;
pub(crate) mod http;

// Provides access to env in native rust environment
pub fn init_env() -> impl EnvIO {
  env::EnvNative::init()
}

// Provides access to file system in native rust environment
pub fn init_file() -> impl FileIO {
  file::NativeFileIO::init()
}

// Provides access to http in native rust environment
pub fn init_http(upstream: &Upstream, http_client_options: &HttpClientOptions) -> impl HttpIO + Default + Clone {
  http::HttpNative::init(upstream, http_client_options)
}
