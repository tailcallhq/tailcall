use anyhow::anyhow;
use url::Url;

use crate::config::{Config, Source};
use crate::io::file::FileIO;
use crate::io::http::HttpIO;

const SUPPORTED_EXT: [&str; 5] = ["json", "yml", "yaml", "graphql", "gql"];

pub struct ConfigReader<File> {
  file: File,
}

impl<File: FileIO> ConfigReader<File> {
  pub fn init(file: File) -> Self {
    Self { file }
  }

  pub async fn read<T: ToString>(&self, files: &[T], http_client: impl HttpIO + Sized) -> anyhow::Result<Config> {
    let files = files.iter().map(|x| x.to_string()).collect::<Vec<String>>();
    let mut config = Config::default();
    for file in files {
      if let Ok(url) = Url::parse(&file) {
        let response = http_client
          .execute_raw(reqwest::Request::new(reqwest::Method::GET, url))
          .await?;
        let sdl = response.headers.get("content-type");
        let sdl = match sdl {
          Some(value) => {
            let value = value.to_str().map_err(|e| anyhow!("{}", e))?.to_string();
            match SUPPORTED_EXT.iter().any(|&substring| value.contains(substring)) {
              true => value,
              false => file.to_string(),
            }
          }
          None => file.to_string(),
        };
        let source = Self::detect_source(&sdl)?;
        let content = String::from_utf8(response.body)?;
        let conf = Config::from_source(source, &content)?;
        config = config.clone().merge_right(&conf);
        continue;
      }
      let (content, path) = self.file.read_file(&file).await?;
      let source = Self::detect_source(&path)?;
      let conf = Config::from_source(source, &content)?;
      config = config.clone().merge_right(&conf);
    }
    Ok(config)
  }

  fn detect_source(source: &str) -> anyhow::Result<Source> {
    let source = if let Ok(s) = Source::detect_content_type(source) {
      s
    } else {
      Source::detect(source.trim_end_matches('/'))?
    };
    Ok(source)
  }
}
#[cfg(test)]
mod reader_tests {
  use tokio::io::AsyncReadExt;

  use crate::config::reader::ConfigReader;
  use crate::config::{Config, Type, Upstream};
  use crate::http::HttpClientOptions;

  fn start_mock_server() -> httpmock::MockServer {
    httpmock::MockServer::start()
  }

  #[tokio::test]
  async fn test_all() {
    let http_client = crate::io::http::init_http_native(&Upstream::default(), &HttpClientOptions::default());
    let mut cfg = Config::default();
    cfg.schema.query = Some("Test".to_string());
    cfg = cfg.types([("Test", Type::default())].to_vec());

    let server = start_mock_server();
    let header_serv = server.mock(|when, then| {
      when.method(httpmock::Method::GET).path("/");
      then
        .status(200)
        .header("content-type", "application/graphql")
        .body(cfg.to_sdl());
    });

    let mut json = String::new();
    tokio::fs::File::open("examples/jsonplaceholder.json")
      .await
      .unwrap()
      .read_to_string(&mut json)
      .await
      .unwrap();

    let foo_json_serv = server.mock(|when, then| {
      when.method(httpmock::Method::GET).path("/foo.json");
      then.status(200).body(json);
    });

    let port = server.port();
    let files: Vec<String> = [
      "examples/jsonplaceholder.yml",                       // config from local file
      format!("http://localhost:{port}/").as_str(),         // with content-type header
      format!("http://localhost:{port}/foo.json").as_str(), // with url extension
    ]
    .iter()
    .map(|x| x.to_string())
    .collect();
    let cr = ConfigReader::init(crate::io::file::init_native());
    let c = cr.read(&files, http_client).await.unwrap();
    assert_eq!(
      ["Post", "Query", "Test", "User"]
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>(),
      c.types.keys().map(|i| i.to_string()).collect::<Vec<String>>()
    );
    foo_json_serv.assert(); // checks if the request was actually made
    header_serv.assert();
  }
  #[tokio::test]
  async fn test_local_files() {
    let http_client = crate::io::http::init_http_native(&Upstream::default(), &HttpClientOptions::default());
    let files: Vec<String> = [
      "examples/jsonplaceholder.yml",
      "examples/jsonplaceholder.graphql",
      "examples/jsonplaceholder.json",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect();
    let cr = ConfigReader::init(crate::io::file::init_native());
    let c = cr.read(&files, http_client).await.unwrap();
    assert_eq!(
      ["Post", "Query", "User"]
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>(),
      c.types.keys().map(|i| i.to_string()).collect::<Vec<String>>()
    );
  }
}
