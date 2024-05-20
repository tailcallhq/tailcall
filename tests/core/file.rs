extern crate core;

use std::path::PathBuf;

use anyhow::{anyhow, Context};
use tailcall::core::FileIO;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::runtime::ExecutionSpec;
pub struct File {
    spec: ExecutionSpec,
}

impl File {
    pub fn new(spec: ExecutionSpec) -> File {
        File { spec }
    }
}

#[async_trait::async_trait]
impl FileIO for File {
    async fn write<'a>(&'a self, _path: &'a str, _content: &'a [u8]) -> anyhow::Result<()> {
        Err(anyhow!("Cannot write to a file in an execution spec"))
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let base = PathBuf::from(path);
        let path = base
            .file_name()
            .context("Invalid file path")?
            .to_str()
            .context("Invalid OsString")?;
        match self.spec.files.get(path) {
            Some(x) => Ok(x.to_owned()),
            None => Err(anyhow!("No such file or directory (os error 2)")),
        }
    }
}

#[derive(Clone)]
pub struct TestFileIO {}

impl TestFileIO {
    pub fn init() -> Self {
        TestFileIO {}
    }
}

#[async_trait::async_trait]
impl FileIO for TestFileIO {
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(content)
            .await
            .map_err(|e| anyhow!("{}", e))?;
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(|e| anyhow!("{}", e))?;
        Ok(String::from_utf8(buffer)?)
    }
}
