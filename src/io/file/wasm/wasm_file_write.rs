use anyhow::{anyhow, Result};

use crate::io::file::FileIO;
impl FileIO {
  pub fn write(_: &str, _: &[u8]) -> Result<()> {
    Err(anyhow!("unimplemented for wasm".to_string()))
  }
}
