use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use tailcall::io::FileIO;
use worker::Env;

pub struct CloudflareFileIO {
  env: Arc<Env>,
  path: PathBuf,
}
unsafe impl Send for CloudflareFileIO {}
unsafe impl Sync for CloudflareFileIO {}
impl CloudflareFileIO {
  pub fn init(path: String, env: Arc<Env>) -> Self {
    let path = PathBuf::from(&path);
    CloudflareFileIO { env, path }
  }
}
#[async_trait::async_trait]
impl FileIO for CloudflareFileIO {
  async fn write<'a>(&'a self, file_path: &'a str, content: &'a [u8]) -> Result<()> {
    let env = self.env.clone();
    let (r2_id, file) = merge_path(self.path.clone(), file_path.to_string()).ok_or(anyhow!("Unexpected path"))?;
    let content = content.to_vec();
    async_std::task::spawn_local(internal_write(env, r2_id, file, content.to_vec())).await?;
    Ok(())
  }

  async fn read<'a>(&'a self, file_path: &'a str) -> Result<(String, String)> {
    let env = self.env.clone();
    let (r2_id, file) = merge_path(self.path.clone(), file_path.to_string()).ok_or(anyhow!("Unexpected path"))?;
    let body = async_std::task::spawn_local(internal_read(env, r2_id, file.clone())).await?;
    Ok((body, file))
  }

  async fn read_all<'a>(&'a self, file_paths: &'a [String]) -> Result<Vec<(String, String)>> {
    let mut vec = Vec::new();
    for file in file_paths {
      vec.push(self.read(file).await.map_err(conv_err)?);
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

fn merge_path(mut head: PathBuf, input: String) -> Option<(String, String)> {
  let path = PathBuf::from(&input);
  if path.is_absolute() {
    return separate_path(path.to_str()?);
  }
  for component in path.components() {
    match component {
      std::path::Component::ParentDir => {
        // for ../
        head.pop();
      }
      std::path::Component::CurDir => (), // for ./
      _ => head.push(component),
    }
  }
  separate_path(head.to_str()?)
}

// Get the bucket id from absolute path (it is expected that all paths starts with bucket id
fn separate_path(input: &str) -> Option<(String, String)> {
  let mut split = input.split("/").filter(|s| !s.is_empty());
  let r2_id = split.next()?.to_string();
  let path = split.collect::<Vec<&str>>().join("/");
  Some((r2_id, path))
}
