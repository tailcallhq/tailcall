use std::rc::Rc;

use anyhow::anyhow;
use tailcall::FileIO;
use worker::Env;

use crate::to_anyhow;

#[derive(Clone)]
pub struct CloudflareFileIO {
  env: Rc<Env>,
  bucket_id: String
}

impl CloudflareFileIO {
  pub fn init(env: Rc<Env>, bucket_id: String) -> Self {
    CloudflareFileIO { env, bucket_id }
  }
}

impl CloudflareFileIO {
  async fn bucket(&self) -> anyhow::Result<worker::Bucket> {
    Ok(self.env.bucket(&self.bucket_id).map_err(|e| anyhow!(e.to_string()))?)
  }

  async fn get(&self, path: String) -> anyhow::Result<String> {
    log::debug!("Reading from bucket:{} path:{}", self.bucket_id, path);
    let bucket = self.bucket().await.map_err(to_anyhow)?;

    let maybe_object = bucket.get(&path).execute().await.map_err(to_anyhow)?;
    let object = maybe_object.ok_or(anyhow!("File {} was not found in bucket: {}", path, self.bucket_id))?;

    let body = match object.body() {
      Some(body) => body.text().await.map_err(to_anyhow),
      None => Ok("".to_string()),
    };
    body
  }

  async fn put(&self, path: String, value: Vec<u8>) -> anyhow::Result<()> {
    let bucket = self.bucket().await.map_err(to_anyhow)?;
    bucket.put(&path, value).execute().await.map_err(to_anyhow)?;
    Ok(())
  }
}

impl FileIO for CloudflareFileIO {
  async fn write<'a>(&'a self, file_path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
    self.put(file_path.to_string(), content.to_vec()).await.map_err(to_anyhow)?;
    Ok(())
  }

  async fn read<'a>(&'a self, file_path: &'a str) -> anyhow::Result<String> {
    let content = self.get(file_path.to_string()).await.map_err(to_anyhow)?;
    Ok(content)
  }
}
