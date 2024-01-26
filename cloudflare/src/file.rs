use std::rc::Rc;

use anyhow::anyhow;
use async_std::task::spawn_local;
use tailcall::FileIO;
use worker::Env;

use crate::to_anyhow;

#[derive(Clone)]
pub struct CloudflareFileIO {
    bucket: Rc<worker::Bucket>,
}

impl CloudflareFileIO {
    pub fn init(env: Rc<Env>, bucket_id: String) -> anyhow::Result<Self> {
        let bucket = env
            .bucket(bucket_id.as_str())
            .map_err(|e| anyhow!(e.to_string()))?;
        let bucket = Rc::new(bucket);
        Ok(CloudflareFileIO { bucket })
    }
}

// TODO: avoid the unsafe impl
unsafe impl Sync for CloudflareFileIO {}

async fn get(bucket: Rc<worker::Bucket>, path: String) -> anyhow::Result<String> {
    let maybe_object = bucket
        .get(path.clone())
        .execute()
        .await
        .map_err(to_anyhow)?;
    let object = maybe_object.ok_or(anyhow!("File '{}' was not found in bucket", path))?;

    let body = match object.body() {
        Some(body) => body.text().await.map_err(to_anyhow),
        None => Ok("".to_string()),
    };
    body
}

async fn put(bucket: Rc<worker::Bucket>, path: String, value: Vec<u8>) -> anyhow::Result<()> {
    bucket.put(path, value).execute().await.map_err(to_anyhow)?;
    Ok(())
}

#[async_trait::async_trait]
impl FileIO for CloudflareFileIO {
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
        let content = content.to_vec();
        let bucket = self.bucket.clone();
        let path_cloned = path.to_string();
        spawn_local(put(bucket, path_cloned, content)).await?;
        log::info!("File write: {} ... ok", path);
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let bucket = self.bucket.clone();
        let path_cloned = path.to_string();
        let content = spawn_local(get(bucket, path_cloned)).await?;
        log::info!("File read: {} ... ok", path);
        Ok(content)
    }
}
