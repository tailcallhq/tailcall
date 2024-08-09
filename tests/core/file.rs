extern crate core;

use std::path::PathBuf;

use tailcall::core::error::file;
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
    async fn write<'a>(&'a self, _path: &'a str, _content: &'a [u8]) -> Result<(), file::Error> {
        Err(file::Error::ExecutionSpecFileWriteFailed)
    }

    async fn read<'a>(&'a self, path: &'a str) -> Result<String, file::Error> {
        let base = PathBuf::from(path);
        let path = base
            .file_name()
            .ok_or(file::Error::InvalidFilePath)?
            .to_str()
            .ok_or(file::Error::InvalidOsString)?;

        match self.spec.files.get(path) {
            Some(x) => Ok(x.to_owned()),
            None => Err(file::Error::NotFound),
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
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> Result<(), file::Error> {
        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(content).await?;
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> Result<String, file::Error> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(String::from_utf8(buffer)?)
    }
}
