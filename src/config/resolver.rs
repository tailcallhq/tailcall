use std::collections::VecDeque;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use url::Url;

use super::super::is_default;
use crate::cli::init_file;
use crate::config::{Config, Source};

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema)]
pub enum LinkType {
    #[default]
    Config,
    GraphQL,
    Protobuf,
    Data,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema)]
pub struct Link {
    ///
    /// Type of the link
    ///
    #[serde(default, skip_serializing_if = "is_default", rename = "type")]
    pub type_of: LinkType,
    ///
    /// Source URL for linked files
    ///
    #[serde(default, skip_serializing_if = "is_default")]
    pub src: String,
    ///
    /// Id is used to refer at different places in the config
    ///
    #[serde(default, skip_serializing_if = "is_default")]
    pub id: Option<String>,
    ///
    /// Raw content of the file
    ///
    content: Option<String>,
}

impl Link {
    fn merge_configs(
        config: Option<Config>,
        config_right: Option<Config>,
    ) -> anyhow::Result<Option<Config>> {
        match (config, config_right) {
            (Some(c), Some(cc)) => Ok(Some(c.merge_right(&cc))),
            (Some(c), None) => Ok(Some(c)),
            (None, Some(cc)) => Ok(Some(cc)),
            (None, None) => Ok(None),
        }
    }

    pub async fn resolve_recurse(config_links: &mut Vec<Link>) -> anyhow::Result<Option<Config>> {
        let mut extend_config_links: Vec<Link> = Vec::new();
        let mut link_queue: VecDeque<Link> = VecDeque::new();
        let mut config = None;

        for config_link in config_links.iter_mut().filter(|link| !link.src.is_empty()) {
            config = Self::merge_configs(
                config,
                Self::resolve_current_link(config_link, &mut link_queue).await?,
            )?;
        }

        while let Some(mut curr_link) = link_queue.pop_front() {
            let current_config =
                Self::resolve_current_link(&mut curr_link, &mut link_queue).await?;
            extend_config_links.push(curr_link);

            config = Self::merge_configs(config, current_config)?;
        }

        config_links.extend(extend_config_links);

        Ok(config)
    }

    async fn resolve_current_link(
        link: &mut Link,
        link_queue: &mut VecDeque<Link>,
    ) -> anyhow::Result<Option<Config>> {
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
        let path = &link.src.trim_end_matches('/');
        let (content, source) = if let Ok(url) = Url::parse(&link.src) {
            let resp = reqwest::get(url).await?;
            if !resp.status().is_success() {
                return Err(anyhow!(
                    "Read over URL failed with status code: {}",
                    resp.status()
                ));
            }
            if link.type_of == LinkType::Config {
                let source = if let Some(v) = resp.headers().get("content-type") {
                    if let Ok(s) = Source::detect(v.to_str()?) {
                        s
                    } else {
                        Source::detect(path)?
                    }
                } else {
                    Source::detect(path)?
                };
                (Some(resp.text().await?), Some(source))
            } else {
                (Some(resp.text().await?), None)
            }
        } else {
            let file = init_file();

            let data = file.read(path).await?;
            if link.type_of == LinkType::Config {
                (Some(data), Some(Source::detect(path)?))
            } else {
                (Some(data), None)
            }
        };
        link.content = content;
        Ok(source)
    }
}
