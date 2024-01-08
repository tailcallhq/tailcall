use std::fs::File;
use std::io::Write;
use crate::io::file::FileIO;
use anyhow::Result;
impl FileIO {
    pub fn write(file: &str, content: &[u8]) -> Result<()> {
        let mut file = File::create(file)?;
        file.write_all(content)?;
        Ok(())
    }
}