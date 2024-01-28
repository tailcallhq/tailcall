use std::sync::Arc;

use anyhow::anyhow;
use tailcall::FileIO;
use tokio::io::AsyncReadExt;

#[derive(Clone, Copy)]
pub struct LambdaFileIO;

#[async_trait::async_trait]
impl FileIO for LambdaFileIO {
    async fn write<'a>(&'a self, _path: &'a str, _content: &'a [u8]) -> anyhow::Result<()> {
        Err(anyhow!("File writing not supported on Lambda."))
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(anyhow::Error::from)?;
        Ok(String::from_utf8(buffer)?)
    }
}

pub fn init_file() -> Arc<LambdaFileIO> {
    Arc::new(LambdaFileIO)
}
