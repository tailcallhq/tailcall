use anyhow::{anyhow, Result};
use url::Url;

use crate::io::file::FileIO;

// This is temporary, should be removed with removal of config for wasm
const SUBSTRINGS: [&'static str; 5] = ["json", "yml", "yaml", "graphql", "gql"];

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
  async fn read_file(file_path: &str) -> Result<(String, String)> {
    let url = Url::parse(file_path)?;
    let response = crate::io::http::get_string(url).await?;
    let sdl = response.headers.get("Content-Type");
    let sdl = match sdl {
      Some(v) => {
        let s = v.to_str().map_err(|e| anyhow!("{}", e))?.to_lowercase();
        if SUBSTRINGS.iter().any(|substring| s.contains(substring)) {
          s.to_string()
        } else {
          file_path.to_string()
        }
      }
      None => file_path.to_string(),
    };
    Ok((response.body, sdl))
  }

  async fn read_files<'a>(&'a self, file_paths: &'a [String]) -> Result<Vec<(String, String)>> {
    let mut files = vec![];
    for file in file_paths {
      let content = Self::read_file(file).await?;
      files.push(content);
    }
    Ok(files)
  }

  async fn write<'a>(_: &'a str, _: &'a [u8]) -> Result<()> {
    unimplemented!("file write operation is not supported in wasm")
  }
}
