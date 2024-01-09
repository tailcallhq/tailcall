use std::collections::VecDeque;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use url::Url;

use crate::config::{is_default, Config, Source};

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
  #[serde(default, skip_serializing_if = "is_default", rename="type")]
  type_of: LinkType, // Type of the link
  #[serde(default, skip_serializing_if = "is_default")]
  src: String, // Source URL for linked files
  #[serde(default, skip_serializing_if = "is_default")]
  pub id: Option<String>, // Id is used to refer at different places in the config
  content: Option<String>, // Stores raw content
}

impl Link {
  pub async fn resolve_recurse(config_links: &mut Vec<Link>) -> anyhow::Result<Option<Config>> {
    let mut extend_config_links: Vec<Link> = Vec::new();
    let mut link_queue: VecDeque<Link> = VecDeque::new();
    let mut config: Option<Config> = None;

    for config_link in config_links.into_iter() {
      config = Self::resolve_current_link(config_link, &mut link_queue).await?;
    }

    while let Some(mut curr_link) = link_queue.pop_front() {
      let current_config = Self::resolve_current_link(&mut curr_link, &mut link_queue).await?;
      extend_config_links.push(curr_link);

      match (config.clone(), current_config) {
        (Some(c), Some(cc)) => config = Some(c.merge_right(&cc)?),
        (Some(c), None) => config = Some(c),
        (None, Some(cc)) => config = Some(cc),
        (None, None) => (),
      }
    }

    config_links.extend(extend_config_links);

    Ok(config)
  }

  async fn resolve_current_link(link: &mut Link, link_queue: &mut VecDeque<Link>) -> anyhow::Result<Option<Config>> {
    let source = Self::get_content(link).await?;
    if link.type_of == LinkType::Config {
      let link_clone = link.clone();
      let config = Config::from_source(source.unwrap(), &link_clone.content.unwrap())?;
      for extended_link in config.links.clone() {
        link_queue.push_back(extended_link);
      }

      return Ok(Some(config));
    }
    Ok(None)
  }

  async fn get_content(link: &mut Link) -> anyhow::Result<Option<Source>> {
    let (content, source) = if let Ok(url) = Url::parse(&link.src) {
      let resp = reqwest::get(url).await?;
      let path = link.src.clone();
      if !resp.status().is_success() {
        return Err(anyhow!("Read over URL failed with status code: {}", resp.status()));
      }
      if link.type_of == LinkType::Config {
        let source = if let Some(v) = resp.headers().get("content-type") {
          if let Ok(s) = Source::detect_content_type(v.to_str()?) {
            s
          } else {
            Source::detect(path.trim_end_matches('/'))?
          }
        } else {
          Source::detect(path.trim_end_matches('/'))?
        };
        (Some(resp.text().await?), Some(source))
      } else {
        (Some(resp.text().await?), None)
      }
    } else {
      let path = &link.src.trim_end_matches('/');
      let mut f = File::open(path).await?;
      let mut buffer: Vec<u8> = Vec::new();
      f.read_to_end(&mut buffer).await?;
      if link.type_of == LinkType::Config {
        (Some(String::from_utf8(buffer)?), Some(Source::detect(path)?))
      } else {
        (Some(String::from_utf8(buffer)?), None)
      }
    };
    link.content = content;
    Ok(source)
  }
}
