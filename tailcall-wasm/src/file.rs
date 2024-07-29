use dashmap::DashMap;
use tailcall::core::error::file;
use tailcall::core::FileIO;

pub struct WasmFile {}

impl WasmFile {
    pub fn init() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl FileIO for WasmFile {
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> file::Result<()> {
        Err(file::Error::FileIONotSupported)
    }

    async fn read<'a>(&'a self, path: &'a str) -> file::Result<String> {
        Err(file::Error::FileIONotSupported)
    }
}
