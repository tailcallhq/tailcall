use std::rc::Rc;

use anyhow::anyhow;
use async_std::task::spawn_local;
use tailcall::core::error::file::FileError;
use tailcall::core::FileIO;
use worker::Env;

use super::{Error, Result};

#[derive(Clone)]
pub struct CloudflareFileIO {
    bucket: Rc<worker::Bucket>,
}

impl CloudflareFileIO {
    pub fn init(env: Rc<Env>, bucket_id: &str) -> anyhow::Result<Self> {
        let bucket = env.bucket(bucket_id).map_err(|e| anyhow!(e.to_string()))?;
        let bucket = Rc::new(bucket);
        Ok(CloudflareFileIO { bucket })
    }
}

// Multi-threading is not enabled in Cloudflare,
// so this doesn't matter, and makes API compliance
// way easier.
unsafe impl Sync for CloudflareFileIO {}
unsafe impl Send for CloudflareFileIO {}

async fn get(bucket: Rc<worker::Bucket>, path: String) -> Result<String> {
    let maybe_object = bucket.get(path.clone()).execute().await?;
    let object = maybe_object.ok_or(Error::MissingFileInBucket(path.to_string()))?;

    let body = match object.body() {
        Some(body) => body.text().await?,
        None => "".to_string(),
    };
    Ok(body)
}

async fn put(bucket: Rc<worker::Bucket>, path: String, value: Vec<u8>) -> Result<()> {
    bucket.put(path, value).execute().await?;
    Ok(())
}

#[async_trait::async_trait]
impl FileIO for CloudflareFileIO {
    type Error = FileError;

    async fn write<'a>(
        &'a self,
        path: &'a str,
        content: &'a [u8],
    ) -> std::result::Result<(), Self::Error> {
        let content = content.to_vec();
        let bucket = self.bucket.clone();
        let path_cloned = path.to_string();
        let _ = spawn_local(put(bucket, path_cloned, content))
            .await
            .map_err(|e| FileError::Cloudflare(e.to_string()));
        tracing::info!("File write: {} ... ok", path);
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> std::result::Result<String, Self::Error> {
        let bucket = self.bucket.clone();
        let path_cloned = path.to_string();
        let content = spawn_local(get(bucket, path_cloned))
            .await
            .map_err(|e| FileError::Cloudflare(e.to_string()));
        tracing::info!("File read: {} ... ok", path);
        content
    }
}
