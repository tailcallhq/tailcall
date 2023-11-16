use std::slice::Iter;

use anyhow::anyhow;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::config::{Config, Source};

pub struct ConfigReader {
  file_paths: Vec<String>,
}

impl ConfigReader {
  pub fn init(file_paths: Iter<String>) -> Self {
    Self { file_paths: file_paths.cloned().collect() }
  }
  pub async fn read(&self) -> anyhow::Result<Config> {
    let mut config = Config::default();
    for path in &self.file_paths {
      let conf = if let Ok(url) = reqwest::Url::parse(path) {
        let (st, source) = Self::read_over_url(url).await?;
        Config::from_source(source, &st)?
      } else {
        let path = path.trim_end_matches('/');
        Self::from_file_path(path).await?
      };
      config = config.clone().merge_right(&conf);
    }
    Ok(config)
  }
  async fn from_file_path(file_path: &str) -> anyhow::Result<Config> {
    let (server_sdl, source) = ConfigReader::read_file(file_path).await?;
    Config::from_source(source, &server_sdl)
  }
  async fn read_file(file_path: &str) -> anyhow::Result<(String, Source)> {
    let mut f = File::open(file_path).await?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).await?;
    Ok((String::from_utf8(buffer)?, Source::detect(file_path)?))
  }
  async fn read_over_url(url: reqwest::Url) -> anyhow::Result<(String, Source)> {
    let path = url.path().to_string();
    let resp = reqwest::get(url).await?;
    if !resp.status().is_success() {
      return Err(anyhow!("Read over URL failed with status code: {}", resp.status()));
    }
    let source = if let Some(v) = resp.headers().get("content-type") {
      if let Ok(s) = Source::detect(v.to_str()?) {
        s
      } else {
        Source::detect(path.trim_end_matches('/'))?
      }
    } else {
      Source::detect(path.trim_end_matches('/'))?
    };
    let txt = resp.text().await?;
    Ok((txt, source))
  }
}
#[cfg(test)]
mod reader_tests {
  use crate::config::reader::ConfigReader;
  static TEST_GQL_BODY: &str = r#"
        schema @server(port: 8000) {
        query: Query
      }

      type Query {
        hello: String! @const(data: "world")
      }
  "#;
  fn start_mock_server() -> mockito::Server {
    mockito::Server::new_with_port(3080)
  }
  #[tokio::test]
  async fn test_all() {
    let mut server = start_mock_server();
    server
      .mock("GET", "/")
      .with_status(200)
      .with_header("content-type", "application/graphql")
      .with_body(TEST_GQL_BODY)
      .create();
    let files: Vec<String> = [
      "examples/jsonplaceholder.yml", // from file
      "http://localhost:3080/",       // with content-type header
      "https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.json", // with url extension
    ]
    .iter()
    .map(|x| x.to_string())
    .collect();
    let cr = ConfigReader::init(files.iter());
    let _ = cr.read().await.unwrap();
  }
}
