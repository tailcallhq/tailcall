use std::rc::Rc;

use anyhow::anyhow;
use tailcall::FileIO;
use worker::Env;

use crate::to_anyhow;

#[derive(Clone)]
pub struct CloudflareFileIO {
  bucket: Rc<worker::Bucket>,
}

impl CloudflareFileIO {
  pub fn init(env: Rc<Env>, bucket_id: String) -> anyhow::Result<Self> {
    let bucket = env.bucket(bucket_id.as_str()).map_err(|e| anyhow!(e.to_string()))?;
    Ok(CloudflareFileIO { bucket: Rc::new(bucket) })
  }
}

impl CloudflareFileIO {
  async fn get(&self, path: String) -> anyhow::Result<String> {
    let maybe_object = self.bucket.get(&path).execute().await.map_err(to_anyhow)?;
    let object = maybe_object.ok_or(anyhow!("File {} was not found in bucket", path))?;

    let body = match object.body() {
      Some(body) => body.text().await.map_err(to_anyhow),
      None => Ok("".to_string()),
    };
    body
  }

  async fn put(&self, path: String, value: Vec<u8>) -> anyhow::Result<()> {
    self.bucket.put(&path, value).execute().await.map_err(to_anyhow)?;
    Ok(())
  }
}

impl FileIO for CloudflareFileIO {
  async fn write<'a>(&'a self, file_path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
    self
      .put(file_path.to_string(), content.to_vec())
      .await
      .map_err(to_anyhow)?;

    log::info!("File write: {} ... ok", file_path);
    Ok(())
  }

  async fn read<'a>(&'a self, file_path: &'a str) -> anyhow::Result<String> {
    let content = self.get(file_path.to_string()).await.map_err(to_anyhow)?;
    log::info!("File read: {} ... ok", file_path);
    Ok(content)
  }
}
