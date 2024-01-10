use std::sync::Arc;

use anyhow::{anyhow, Result};
use tailcall::io::FileIO;
use worker::Env;

pub struct CloudflareFileIO {
  env: Arc<Env>,
  r2_id: String,
}
unsafe impl Send for CloudflareFileIO {}
unsafe impl Sync for CloudflareFileIO {}
impl CloudflareFileIO {
  pub fn init(r2_id: String, env: Arc<Env>) -> Self {
    CloudflareFileIO { env, r2_id }
  }
}
#[async_trait::async_trait]
impl FileIO for CloudflareFileIO {
  async fn write<'a>(&'a self, file: &'a str, content: &'a [u8]) -> Result<()> {
    let env = self.env.clone();
    let r2 = self.r2_id.clone();
    let file = file.to_string();
    let content = content.to_vec();
    async_std::task::spawn_local(internal_write(env, r2, file, content.to_vec())).await?;
    Ok(())
  }

  async fn read_file<'a>(&'a self, file: &'a str) -> Result<(String, String)> {
    let env = self.env.clone();
    let r2_id = self.r2_id.clone();
    let file = file.to_string();
    let body = async_std::task::spawn_local(internal_read(env, r2_id, file.clone())).await?;
    Ok((body, file.to_string()))
  }

  async fn read_files<'a>(&'a self, files: &'a [String]) -> Result<Vec<(String, String)>> {
    let mut vec = Vec::new();
    for file in files {
      vec.push(self.read_file(file).await.map_err(conv_err)?);
    }
    Ok(vec)
  }
}

async fn internal_read(env: Arc<Env>, r2_id: String, file: String) -> Result<String> {
  let object = env
    .bucket(&r2_id)
    .map_err(conv_err)?
    .get(&file)
    .execute()
    .await
    .map_err(conv_err)?
    .ok_or(anyhow!("File: {file} not found"))?;
  let body = object.body().ok_or(anyhow!("File: {file} not found"))?;
  let body = body.text().await.map_err(conv_err)?;
  Ok(body)
}

async fn internal_write(env: Arc<Env>, r2_id: String, file: String, content: Vec<u8>) -> Result<()> {
  env
    .bucket(&r2_id)
    .map_err(conv_err)?
    .put(file, content)
    .execute()
    .await
    .map_err(conv_err)?;
  Ok(())
}

fn conv_err<T: std::fmt::Display>(e: T) -> anyhow::Error {
  anyhow!("{}", e)
}
