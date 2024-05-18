use anyhow::anyhow;
use dashmap::DashMap;
use tailcall::core::FileIO;

pub struct WasmFile {
    files: DashMap<String, String>,
}

impl WasmFile {
    pub fn init() -> Self {
        Self { files: DashMap::new() }
    }
}

#[async_trait::async_trait]
impl FileIO for WasmFile {
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
        self.files
            .insert(path.to_string(), String::from_utf8(content.to_vec())?);
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let content = self
            .files
            .get(path)
            .ok_or(anyhow!("File not found"))?
            .value()
            .clone();
        Ok(content)
    }
}
