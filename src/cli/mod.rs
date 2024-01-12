mod command;
mod error;
mod fmt;
pub mod opentelemetry;
pub mod server;
mod tc;

pub use error::CLIError;
pub use tc::run;

use crate::config::Upstream;
use crate::{FileIO, HttpIO};

pub(crate) mod env;
pub(crate) mod file;
pub(crate) mod http;
pub use env::EnvNative;
pub use file::NativeFileIO;
pub use http::HttpNative;

// Provides access to env in native rust environment
pub fn init_env() -> env::EnvNative {
  env::EnvNative::init()
}

// Provides access to file system in native rust environment
pub fn init_file() -> impl FileIO {
  file::NativeFileIO::init()
}

// Provides access to http in native rust environment
pub fn init_http(upstream: &Upstream) -> http::HttpNative {
  http::HttpNative::init(upstream)
}

// Provides access to http in native rust environment
pub fn init_http2_only(upstream: &Upstream) -> http::HttpNative {
  http::HttpNative::init(&upstream.clone().http2_only(true))
}
