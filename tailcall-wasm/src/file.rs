use anyhow::anyhow;
use dashmap::DashMap;
use tailcall::core::FileIO;

pub struct WasmFile {}

impl WasmFile {
    pub fn init() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl FileIO for WasmFile {
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("File IO is not supported"))
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        Err(anyhow::anyhow!("File IO is not supported"))
    }
}
