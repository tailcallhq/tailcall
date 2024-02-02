use std::collections::HashMap;
use std::sync::Arc;
use anyhow::anyhow;
use async_trait::async_trait;
use hyper::body::Bytes;
use reqwest::{Client, Request};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tailcall::{EnvIO, HttpIO};
use tailcall::cache::InMemoryCache;
use tailcall::http::Response;
use tailcall::target_runtime::TargetRuntime;

pub struct Env {
    env: HashMap<String, String>,
}

#[derive(Clone)]
pub struct FileIO {}

impl FileIO {
    pub fn init() -> Self {
        FileIO {}
    }
}

#[async_trait::async_trait]
impl tailcall::FileIO for FileIO {
    async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(content).await.map_err(|e|anyhow!("{}",e))?;
        log::info!("File write: {} ... ok", path);
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(|e|anyhow!("{}",e))?;
        log::info!("File read: {} ... ok", path);
        Ok(String::from_utf8(buffer)?)
    }
}


impl EnvIO for Env {
    fn get(&self, key: &str) -> Option<String> {
        self.env.get(key).cloned()
    }
}

impl Env {
    pub fn init(map: HashMap<String, String>) -> Self {
        Self { env: map }
    }
}

struct Http {
    client: Client
}
#[async_trait]
impl HttpIO for Http {
    async fn execute(&self, request: Request) -> anyhow::Result<Response<Bytes>> {
        let resp = self.client.execute(request).await?;
        let resp = tailcall::http::Response::from_reqwest(resp).await?;
        Ok(resp)
    }
}

fn init_runtime() -> TargetRuntime {
    let http = Arc::new(Http{ client: Client::new() });
    let http2_only = http.clone();
    TargetRuntime {
        http,
        http2_only,
        env: Arc::new(Env::init(HashMap::new())),
        file: Arc::new(FileIO::init()),
        cache: Arc::new(InMemoryCache::new()),
    }
}
#[cfg(test)]
mod reader_tests {
    use anyhow::Context;
    use pretty_assertions::assert_eq;
    use tokio::io::AsyncReadExt;

    use tailcall::config::reader::ConfigReader;
    use tailcall::config::{Config, Script, ScriptOptions, Type};
    use crate::init_runtime;

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    #[tokio::test]
    async fn test_all() {
        let runtime = init_runtime();

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
        let cr = ConfigReader::init(runtime);
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
        let runtime = init_runtime();

        let files: Vec<String> = [
            "examples/jsonplaceholder.yml",
            "examples/jsonplaceholder.graphql",
            "examples/jsonplaceholder.json",
        ]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let cr = ConfigReader::init(runtime);
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
        let runtime = init_runtime();

        let cargo_manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let reader = ConfigReader::init(runtime);

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
