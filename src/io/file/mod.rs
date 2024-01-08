#[cfg(feature = "default")]
pub mod cli;
#[cfg(not(feature = "default"))]
pub mod wasm;

#[cfg(feature = "default")]
pub use cli::*;
#[cfg(not(feature = "default"))]
pub use wasm::*;

#[derive(Default)]
pub struct FileIO {
  files: Vec<String>,
}

impl FileIO {
  pub fn init<Iter>(file_paths: Iter) -> Self
  where
    Iter: Iterator,
    Iter::Item: AsRef<str>,
  {
    Self { files: file_paths.map(|path| path.as_ref().to_owned()).collect() }
  }
}
