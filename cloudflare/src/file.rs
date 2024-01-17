use std::rc::Rc;

use anyhow::anyhow;
use tailcall::FileIO;
use worker::Env;

use crate::to_anyhow;

#[derive(Clone)]
pub struct CloudflareStaticFileIO {
  env: Rc<Env>,
}

impl CloudflareStaticFileIO {
  pub fn init(env: Rc<Env>) -> Self {
    CloudflareStaticFileIO { env }
  }
}

impl CloudflareStaticFileIO {
  async fn kv_store(&self) -> anyhow::Result<worker::kv::KvStore> {
    Ok(self.env.kv("__STATIC_CONTENT").map_err(|e| anyhow!(e.to_string()))?)
  }

  async fn get(&self, path: String) -> anyhow::Result<String> {
    log::debug!("Reading from kv store: STATIC_CONTENT path:{}", path);
    let kv_store = self.kv_store().await.map_err(to_anyhow)?;
    kv_store
      .get(&path)
      .text()
      .await
      .map(|result| result.unwrap_or_default())
      .map_err(to_anyhow)
  }

  async fn put(&self, path: String, value: Vec<u8>) -> anyhow::Result<()> {
    let kv_store = self.kv_store().await.map_err(to_anyhow)?;
    kv_store
      .put(&path, value)
      .map_err(to_anyhow)?
      .execute()
      .await
      .map_err(to_anyhow)?;
    Ok(())
  }
}

impl FileIO for CloudflareStaticFileIO {
  async fn write<'a>(&'a self, file_path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
    self
      .put(file_path.to_string(), content.to_vec())
      .await
      .map_err(to_anyhow)?;
    Ok(())
  }

  async fn read<'a>(&'a self, file_path: &'a str) -> anyhow::Result<String> {
    let content = self.get(file_path.to_string()).await.map_err(to_anyhow)?;
    Ok(content)
  }
}

#[derive(Clone)]
pub struct CloudflareR2FileIO {
  env: Rc<Env>,
  id: String,
}

impl CloudflareR2FileIO {
  pub fn init(env: Rc<Env>, id: String) -> Self {
    CloudflareR2FileIO { env, id }
  }
}

impl CloudflareR2FileIO {
  async fn bucket(&self) -> anyhow::Result<worker::Bucket> {
    Ok(self.env.bucket(&self.id).map_err(|e| anyhow!(e.to_string()))?)
  }

  async fn get(&self, path: String) -> anyhow::Result<String> {
    log::debug!("Reading from bucket:{} path:{}", self.id, path);
    let bucket = self.bucket().await.map_err(to_anyhow)?;

    let maybe_object = bucket.get(&path).execute().await.map_err(to_anyhow)?;
    let object = maybe_object.ok_or(anyhow!("File {} was not found in bucket: {}", path, self.id))?;

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

impl FileIO for CloudflareR2FileIO {
  async fn write<'a>(&'a self, file_path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
    self
      .put(file_path.to_string(), content.to_vec())
      .await
      .map_err(to_anyhow)?;
    Ok(())
  }

  async fn read<'a>(&'a self, file_path: &'a str) -> anyhow::Result<String> {
    let content = self.get(file_path.to_string()).await.map_err(to_anyhow)?;
    Ok(content)
  }
}
