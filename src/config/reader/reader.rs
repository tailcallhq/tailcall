#[cfg(feature = "default")]
pub use super::reader_cli::*;
#[cfg(not(feature = "default"))]
pub use super::reader_wasm::*;

impl ConfigReader {
  pub fn init<Iter>(file_paths: Iter) -> Self
  where
    Iter: Iterator,
    Iter::Item: AsRef<str>,
  {
    Self { file_paths: file_paths.map(|path| path.as_ref().to_owned()).collect() }
  }
}
