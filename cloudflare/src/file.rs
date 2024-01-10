use std::sync::Arc;

use anyhow::{anyhow, Result};
use tailcall::io::FileIO;
use worker::{Bucket, Env};

pub struct CloudflareFileIO {
  bucket: Arc<Bucket>,
}
unsafe impl Send for CloudflareFileIO {}
unsafe impl Sync for CloudflareFileIO {}
impl CloudflareFileIO {
  pub fn init(r2_id: &str, env: Arc<Env>) -> Result<Self> {
    let bucket = env.bucket(r2_id).map_err(conv_err)?;
    let bucket = Arc::new(bucket);
    Ok(CloudflareFileIO { bucket })
  }
}
// FIXME fix the errors in the methods
#[async_trait::async_trait]
impl FileIO for CloudflareFileIO {
  async fn write<'a>(&'a self, file: &'a str, content: &'a [u8]) -> Result<()> {
    self
      .bucket
      .put(file, content.to_vec())
      .execute()
      .await
      .map_err(conv_err)?;
    Ok(())
  }

  async fn read_file<'a>(&'a self, _: &'a str) -> Result<(String, String)> {
    unimplemented!("file read I/O is not required for cloudflare")
  }

  async fn read_files<'a>(&'a self, _: &'a [String]) -> Result<Vec<(String, String)>> {
    unimplemented!("file read I/O is not required for cloudflare")
  }
}
