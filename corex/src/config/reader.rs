use futures_util::future::join_all;
use url::Url;

use crate::config::{Config, Source};
use crate::{FileIO, HttpIO};

/// Reads the configuration from a file or from an HTTP URL and resolves all linked assets.
pub struct ConfigReader<File, Http> {
  file: File,
  http: Http,
}

struct FileRead {
  content: String,
  path: String,
}

impl<File: FileIO, Http: HttpIO> ConfigReader<File, Http> {
  pub fn init(file: File, http: Http) -> Self {
    Self { file, http }
  }

  /// Reads a file from the filesystem or from an HTTP URL
  async fn read_file<T: ToString>(&self, file: T) -> anyhow::Result<FileRead> {
    // Is an HTTP URL
    let content = if let Ok(url) = Url::parse(&file.to_string()) {
      let response = self
        .http
        .execute(reqwest::Request::new(reqwest::Method::GET, url))
        .await?;

      String::from_utf8(response.body.to_vec())?
    } else {
      // Is a file path
      self.file.read(&file.to_string()).await?
    };

    Ok(FileRead { content, path: file.to_string() })
  }

  /// Reads all the files in parallel
  async fn read_files<T: ToString>(&self, files: &[T]) -> anyhow::Result<Vec<FileRead>> {
    let files = files.iter().map(|x| self.read_file(x.to_string()));
    let content = join_all(files).await.into_iter().collect::<anyhow::Result<Vec<_>>>()?;
    Ok(content)
  }

  pub async fn read<T: ToString>(&self, files: &[T]) -> anyhow::Result<Config> {
    let files = self.read_files(files).await?;
    let mut config = Config::default();
    for file in files.iter() {
      let source = Source::detect(&file.path)?;
      let schema = &file.content;
      let new_config = Config::from_source(source, schema)?;
      config = config.merge_right(&new_config);
    }

    Ok(config)
  }
}
