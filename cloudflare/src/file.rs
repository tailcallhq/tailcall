use std::rc::Rc;

use anyhow::anyhow;
use tailcall::FileIO;
use worker::Env;

use crate::r2_address::R2Address;
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
}

impl CloudflareR2FileIO {
  pub fn init(env: Rc<Env>) -> Self {
    CloudflareR2FileIO { env }
  }
}

impl CloudflareR2FileIO {
  async fn bucket(&self, r2: &R2Address) -> anyhow::Result<worker::Bucket> {
    Ok(self.env.bucket(&r2.bucket).map_err(|e| anyhow!(e.to_string()))?)
  }

  async fn get(&self, r2: &R2Address) -> anyhow::Result<String> {
    log::debug!("Reading from bucket:{} path:{}", r2.bucket, r2.path);
    let bucket = self.bucket(&r2).await.map_err(to_anyhow)?;

    let maybe_object = bucket.get(&r2.path).execute().await.map_err(to_anyhow)?;
    let object = maybe_object.ok_or(anyhow!("File {} was not found in bucket: {}", r2.path, r2.bucket))?;

    let body = match object.body() {
      Some(body) => body.text().await.map_err(to_anyhow),
      None => Ok("".to_string()),
    };
    body
  }

  async fn put(&self, r2: &R2Address, value: Vec<u8>) -> anyhow::Result<()> {
    let bucket = self.bucket(&r2).await.map_err(to_anyhow)?;
    bucket.put(&r2.path, value).execute().await.map_err(to_anyhow)?;
    Ok(())
  }
}

impl FileIO for CloudflareR2FileIO {
  async fn write<'a>(&'a self, file_path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
    let r2 = R2Address::from_string(file_path.to_string())?;
    self.put(&r2, content.to_vec()).await.map_err(to_anyhow)?;
    Ok(())
  }

  async fn read<'a>(&'a self, file_path: &'a str) -> anyhow::Result<String> {
    let r2 = R2Address::from_string(file_path.to_string())?;
    let content = self.get(&r2).await.map_err(to_anyhow)?;
    Ok(content)
  }
}
