use anyhow::Result;

use crate::io::file::FileIO;
pub struct WasmFileIO {}

impl WasmFileIO {
  pub fn init() -> Self {
    WasmFileIO {}
  }
}

// TODO: Temporary implementation that performs an HTTP request to get the file content
// This should be moved to a more native implementation that's based on the WASM env.
#[async_trait::async_trait]
impl FileIO for WasmFileIO {
  async fn write<'a>(&'a self, file: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
    unimplemented!("file write I/O is not required for cloudflare")
  }

  async fn read_file<'a>(&'a self, _: &'a str) -> Result<(String, String)> {
    unimplemented!("file read I/O is not required for cloudflare")
  }

  async fn read_files<'a>(&'a self, _: &'a [String]) -> Result<Vec<(String, String)>> {
    unimplemented!("file read I/O is not required for cloudflare")
  }
}
