use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use hyper::body::Bytes;
use reqwest::{Client, Request};
use tailcall::cache::InMemoryCache;
use tailcall::http::Response;
use tailcall::target_runtime::TargetRuntime;
use tailcall::{EnvIO, HttpIO};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
        file.write_all(content)
            .await
            .map_err(|e| anyhow!("{}", e))?;
        log::info!("File write: {} ... ok", path);
        Ok(())
    }

    async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(|e| anyhow!("{}", e))?;
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
    client: Client,
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
    let http = Arc::new(Http { client: Client::new() });
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
mod serv_spec {
    use reqwest::Client;
    use serde_json::json;
    use tailcall::cli::server::Server;
    use tailcall::config::reader::ConfigReader;

    use crate::init_runtime;

    async fn test_server(configs: &[&str], url: &str) {
        let runtime = init_runtime();
        let reader = ConfigReader::init(runtime);
        let config = reader.read_all(configs).await.unwrap();
        let mut server = Server::new(config);
        let server_up_receiver = server.server_up_receiver();

        tokio::spawn(async move {
            server.start().await.unwrap();
        });

        server_up_receiver
            .await
            .expect("Server did not start up correctly");

        // required since our cert is self signed
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();
        let query = json!({
            "query": "{ greet }"
        });

        let mut tasks = vec![];
        for _ in 0..100 {
            let client = client.clone();
            let url = url.to_owned();
            let query = query.clone();

            let task: tokio::task::JoinHandle<Result<serde_json::Value, anyhow::Error>> =
                tokio::spawn(async move {
                    let response = client.post(url).json(&query).send().await?;
                    let response_body: serde_json::Value = response.json().await?;
                    Ok(response_body)
                });
            tasks.push(task);
        }

        for task in tasks {
            let response_body = task
                .await
                .expect("Spawned task should success")
                .expect("Request should success");
            let expected_response = json!({
                "data": {
                    "greet": "Hello World!"
                }
            });
            assert_eq!(
                response_body, expected_response,
                "Unexpected response from server"
            );
        }
    }

    #[tokio::test]
    async fn server_start() {
        test_server(
            &["tests/server/config/server-start.graphql"],
            "http://localhost:8800/graphql",
        )
        .await
    }

    #[tokio::test]
    async fn server_start_http2_pcks8() {
        test_server(
            &["tests/server/config/server-start-http2-pkcs8.graphql"],
            "https://localhost:8801/graphql",
        )
        .await
    }

    #[tokio::test]
    async fn server_start_http2_rsa() {
        test_server(
            &["tests/server/config/server-start-http2-rsa.graphql"],
            "https://localhost:8802/graphql",
        )
        .await
    }

    #[tokio::test]
    async fn server_start_http2_nokey() {
        let configs = &["tests/server/config/server-start-http2-nokey.graphql"];
        let runtime = init_runtime();
        let reader = ConfigReader::init(runtime);
        let config = reader.read_all(configs).await.unwrap();
        let server = Server::new(config);
        assert!(server.start().await.is_err())
    }

    #[tokio::test]
    async fn server_start_http2_ec() {
        test_server(
            &["tests/server/config/server-start-http2-ec.graphql"],
            "https://localhost:8804/graphql",
        )
        .await
    }
}
