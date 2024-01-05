use std::collections::VecDeque;

use crate::config::is_default;
use crate::config::{Config, Source};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use url::Url;

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
  pub async fn resolve_recurse(self) -> anyhow::Result<Vec<Link>> {
    let mut link_queue: VecDeque<Link> = VecDeque::new();
    let mut result: Vec<Link> = Vec::new();
    link_queue.push_back(self);
    while let Some(mut curr) = link_queue.pop_front() {
      let (txt, source) = if let Ok(url) = Url::parse(&curr.src) {
        let resp = reqwest::get(url).await?;
        let path = curr.src.clone();
        if !resp.status().is_success() {
          return Err(anyhow!("Read over URL failed with status code: {}", resp.status()));
        }
        let source = if let Some(v) = resp.headers().get("content-type") {
          if let Ok(s) = Source::detect_content_type(v.to_str()?) {
            s
          } else {
            Source::detect(path.trim_end_matches('/'))?
          }
        } else {
          Source::detect(path.trim_end_matches('/'))?
        };
        (resp.text().await?, source)
      } else {
        let path = &curr.src.trim_end_matches('/');
        let mut f = File::open(path).await?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).await?;
        (String::from_utf8(buffer)?, Source::detect(path)?)
      };

      curr.content = Some(txt.clone());
      let config = Config::from_source(source, &txt)?;

      for link in config.links {
        link_queue.push_back(link);
      }

      result.push(curr);
    }
    Ok(result)
  }
}
