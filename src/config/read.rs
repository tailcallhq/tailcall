use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::config::{Config, Source};

pub struct ConfigReader {
    config: Config,
}

impl ConfigReader {
    pub fn init() -> Self {
        Self { config: Config::default() }
    }
    pub async fn serialize_config(&mut self, path: &String) -> anyhow::Result<()> {
        let conf = if path.starts_with("http://") || path.starts_with("https://") {
            let (st, source) = Self::read_over_url(path).await?;
            Config::from_source(source, &st)? // needs improvement
        } else {
            Config::from_file_path(path).await?
        };
        self.config = self.config.clone().merge_right(&conf);
        Ok(())
    }
    pub async fn read_file(file_path: &String) -> anyhow::Result<String> {
        let mut f = File::open(file_path).await?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).await?;
        Ok(String::from_utf8(buffer)?)
    }
    async fn read_over_url(path: &String) -> anyhow::Result<(String, Source)> {
        let resp = reqwest::get(path).await?;
        let source = if let Some(v) = resp.headers().get("content-type") {
            if let Ok(s) = Source::detect(v.to_str()?) {
                s
            }else {
                Source::detect(path)?
            }
        }else {
            Source::detect(path)?
        };
        let txt = resp.text().await?;
        Ok((txt,source))
    }
    pub fn get_config(&self) -> &Config {
        &self.config
    }
}