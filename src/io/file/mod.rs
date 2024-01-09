#[cfg(feature = "default")]
pub mod native;

#[cfg(not(feature = "default"))]
pub mod cloudflare;

#[async_trait::async_trait]
pub trait FileIO {
  async fn write<'a>(file: &'a str, content: &'a [u8]) -> anyhow::Result<()>;
  async fn read_file<'a>(&'a self, file_path: &'a str) -> anyhow::Result<(String, String)>;
  async fn read_files<'a>(&'a self, file_paths: &'a [String]) -> anyhow::Result<Vec<(String, String)>>;
}

#[cfg(not(feature = "default"))]
pub fn init_cloudflare() -> impl FileIO {
  cloudflare::WasmFileIO::init()
}

#[cfg(feature = "default")]
pub fn init_native() -> impl FileIO {
  native::NativeFileIO::init()
}
