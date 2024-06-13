extern crate core;

use std::path::PathBuf;

use tailcall::core::error::file::FileError;
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
    type Error = FileError;
    async fn write<'a>(&'a self, _path: &'a str, _content: &'a [u8]) -> Result<(), Self::Error> {
        Err(FileError::ExecutionSpecFileWriteFailed)
    }

    async fn read<'a>(&'a self, path: &'a str) -> Result<String, Self::Error> {
        let base = PathBuf::from(path);
        let path = base
            .file_name()
            .ok_or(FileError::InvalidFilePath)?
            .to_str()
            .ok_or(FileError::InvalidOsString)?;

        match self.spec.files.get(path) {
            Some(x) => Ok(x.to_owned()),
            None => Err(FileError::NotFound),
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
    type Error = FileError;
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> Result<(), Self::Error> {
        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(content).await?;
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> Result<String, Self::Error> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(String::from_utf8(buffer)?)
    }
}
