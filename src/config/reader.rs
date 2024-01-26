use std::sync::Arc;

use futures_util::future::join_all;
use futures_util::TryFutureExt;
use url::Url;

use super::{Script, ScriptOptions};
use crate::config::{Config, Source};
use crate::{FileIO, HttpIO};

/// Reads the configuration from a file or from an HTTP URL and resolves all linked assets.
pub struct ConfigReader {
    file: Arc<dyn FileIO + Send>,
    http: Arc<dyn HttpIO + Send + Sync>,
}

struct FileRead {
    content: String,
    path: String,
}

impl ConfigReader {
    pub fn init(file: Arc<dyn FileIO + Send + Sync>, http: Arc<dyn HttpIO + Send + Sync>) -> Self {
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
        let files = files.iter().map(|x| {
            self.read_file(x.to_string())
                .map_err(|e| e.context(x.to_string()))
        });
        let content = join_all(files)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(content)
    }

    /// Reads the script file and replaces the path with the content
    async fn read_script(&self, mut config: Config) -> anyhow::Result<Config> {
        if let Some(Script::Path(options)) = config.server.script {
            let timeout = options.timeout;
            let path = options.src;
            let script = self.read_file(path.clone()).await?.content;
            config.server.script = Some(Script::File(ScriptOptions { src: script, timeout }));
        }
        Ok(config)
    }

    /// Reads a single file and returns the config
    pub async fn read<T: ToString>(&self, file: T) -> anyhow::Result<Config> {
        self.read_all(&[file]).await
    }

    /// Reads all the files and returns a merged config
    pub async fn read_all<T: ToString>(&self, files: &[T]) -> anyhow::Result<Config> {
        let files = self.read_files(files).await?;
        let mut config = Config::default();
        for file in files.iter() {
            let source = Source::detect(&file.path)?;
            let schema = &file.content;
            let new_config = Config::from_source(source, schema)?;
            let new_config = self.read_script(new_config).await?;
            config = config.merge_right(&new_config);
        }

        Ok(config)
    }
}

#[cfg(test)]
mod reader_tests {
    use anyhow::Context;
    use pretty_assertions::assert_eq;
    use tokio::io::AsyncReadExt;

    use crate::cli::{init_file, init_http};
    use crate::config::reader::ConfigReader;
    use crate::config::{Config, Script, ScriptOptions, Type, Upstream};

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    #[tokio::test]
    async fn test_all() {
        let mut cfg = Config::default();
        cfg.schema.query = Some("Test".to_string());
        cfg = cfg.types([("Test", Type::default())].to_vec());

        let server = start_mock_server();
        let header_serv = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/bar.graphql");
            then.status(200).body(cfg.to_sdl());
        });

        let mut json = String::new();
        tokio::fs::File::open("examples/jsonplaceholder.json")
            .await
            .unwrap()
            .read_to_string(&mut json)
            .await
            .unwrap();

        let foo_json_server = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/foo.json");
            then.status(200).body(json);
        });

        let port = server.port();
        let files: Vec<String> = [
            "examples/jsonplaceholder.yml", // config from local file
            format!("http://localhost:{port}/bar.graphql").as_str(), // with content-type header
            format!("http://localhost:{port}/foo.json").as_str(), // with url extension
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        let cr = ConfigReader::init(init_file(), init_http(&Upstream::default(), None));
        let c = cr.read_all(&files).await.unwrap();
        assert_eq!(
            ["Post", "Query", "Test", "User"]
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>(),
            c.types
                .keys()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        );
        foo_json_server.assert(); // checks if the request was actually made
        header_serv.assert();
    }

    #[tokio::test]
    async fn test_local_files() {
        let files: Vec<String> = [
            "examples/jsonplaceholder.yml",
            "examples/jsonplaceholder.graphql",
            "examples/jsonplaceholder.json",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        let cr = ConfigReader::init(init_file(), init_http(&Upstream::default(), None));
        let c = cr.read_all(&files).await.unwrap();
        assert_eq!(
            ["Post", "Query", "User"]
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>(),
            c.types
                .keys()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        );
    }

    #[tokio::test]
    async fn test_script_loader() {
        let cargo_manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let reader = ConfigReader::init(init_file(), init_http(&Upstream::default(), None));

        let config = reader
            .read(&format!(
                "{}/examples/jsonplaceholder_script.graphql",
                cargo_manifest
            ))
            .await
            .unwrap();

        let path = format!("{}/examples/scripts/echo.js", cargo_manifest);
        let file = ScriptOptions {
            src: String::from_utf8(
                tokio::fs::read(&path)
                    .await
                    .context(path.to_string())
                    .unwrap(),
            )
            .unwrap(),
            timeout: None,
        };
        assert_eq!(config.server.script, Some(Script::File(file)),);
    }
}
