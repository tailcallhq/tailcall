use std::rc::Rc;

use anyhow::anyhow;
use tailcall::FileIO;
use worker::Env;

use crate::to_anyhow;

#[derive(Clone)]
pub struct CloudflareFileIO {
  bucket: Rc<worker::Bucket>,
}

unsafe impl Send for CloudflareFileIO {}
unsafe impl Sync for CloudflareFileIO {}

impl CloudflareFileIO {
  pub fn init(env: Rc<Env>, bucket_id: String) -> anyhow::Result<Self> {
    let bucket = env.bucket(bucket_id.as_str()).map_err(|e| anyhow!(e.to_string()))?;
    Ok(CloudflareFileIO { bucket: Rc::new(bucket) })
  }
}

impl CloudflareFileIO {}

#[async_trait::async_trait]
impl FileIO for CloudflareFileIO {
  async fn write<'a>(&'a self, file_path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
    let path = file_path.to_string();
    let value = content.to_vec();
    let bucket = self.bucket.clone();

    async_std::task::spawn_local(async move {
      bucket.put(&path, value).execute().await.map_err(to_anyhow)?;
      anyhow::Ok(())
    })
    .await?;

    log::info!("File write: {} ... ok", file_path);
    Ok(())
  }

  async fn read<'a>(&'a self, file_path: &'a str) -> anyhow::Result<String> {
    let bucket = self.bucket.clone();
    let path = file_path.to_string();
    let content = async_std::task::spawn_local(async move {
      let maybe_object = bucket.get(&path).execute().await.map_err(to_anyhow)?;
      let object = maybe_object.ok_or(anyhow!("File '{}' was not found in bucket", path))?;

      let body = match object.body() {
        Some(body) => body.text().await.map_err(to_anyhow),
        None => anyhow::Ok("".to_string()),
      };
      body
    })
    .await?;
    log::info!("File read: {} ... ok", file_path);
    Ok(content)
  }
}
