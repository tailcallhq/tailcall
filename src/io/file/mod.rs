#[cfg(feature = "default")]
pub mod native_impl;
#[cfg(not(feature = "default"))]
pub mod wasm;

#[cfg(feature = "default")]
pub use native_impl::*;

#[cfg(not(feature = "default"))]
pub use wasm::*;

pub struct FileIO {}

impl FileIO {
  pub fn init() -> Self {
    FileIO {}
  }
}
