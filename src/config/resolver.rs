use serde::{Deserialize, Serialize};
use url::Url;
use crate::config::is_default;
use anyhow::anyhow;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
enum LinkType {
  #[default]
  Config,
  GraphQL,
  Protobuf,
  Data,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Link {
  #[serde(default, skip_serializing_if = "is_default")]
  type_of: LinkType, // Type of the link
  #[serde(default, skip_serializing_if = "is_default")]
  src: String, // Source URL for linked files
  #[serde(default, skip_serializing_if = "is_default")]
  id: Option<String>, // Id is used to refer at different places in the config
  #[serde(default, skip_serializing_if = "is_default")]
  content: Option<String>, // Stores raw content
}

impl Link {

  pub async fn resolve_recurse(self) -> anyhow::Result<Link> {
    let mut link = self.clone();
    if let Ok(url) = Url::parse(&self.src) {
      let resp = reqwest::get(url).await?;
      if !resp.status().is_success() {
        return Err(anyhow!("Read over URL failed with status code: {}", resp.status()));
      }
      link.content = Some(resp.text().await?);
    } else {
      let path = &self.src.trim_end_matches('/');
      let mut f = File::open(path).await?;
      let mut buffer = Vec::new();
      f.read_to_end(&mut buffer).await?;
      link.content = Some(String::from_utf8(buffer)?);
    };
    Ok(link)
  }

}
