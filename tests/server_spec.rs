#[cfg(test)]
pub mod test {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use anyhow::{anyhow, Result};
    use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
    use hyper::body::Bytes;
    use reqwest::Client;
    use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
    use tailcall::cache::InMemoryCache;
    use tailcall::http::Response;
    use tailcall::runtime::TargetRuntime;
    use tailcall::{EnvIO, FileIO, HttpIO};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[derive(Clone)]
    struct TestHttp {
        client: ClientWithMiddleware,
    }

    impl Default for TestHttp {
        fn default() -> Self {
            Self { client: ClientBuilder::new(Client::new()).build() }
        }
    }

    impl TestHttp {
        fn init(h2only: bool) -> Self {
            let mut builder = Client::builder()
                .tcp_keepalive(Some(Duration::from_secs(5)))
                .timeout(Duration::from_secs(60))
                .connect_timeout(Duration::from_secs(60))
                .http2_keep_alive_interval(Some(Duration::from_secs(60)))
                .http2_keep_alive_timeout(Duration::from_secs(60))
                .http2_keep_alive_while_idle(false)
                .pool_idle_timeout(Some(Duration::from_secs(60)))
                .pool_max_idle_per_host(60)
                .user_agent("Tailcall/1.0".to_string());

            // Add Http2 Prior Knowledge
            if h2only {
                builder = builder.http2_prior_knowledge();
            }

            let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));

            client = client.with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: MokaManager::default(),
                options: HttpCacheOptions::default(),
            }));

            Self { client: client.build() }
        }
    }

    #[async_trait::async_trait]
    impl HttpIO for TestHttp {
        async fn execute(&self, request: reqwest::Request) -> Result<Response<Bytes>> {
            let response = self.client.execute(request).await;
            Response::from_reqwest(response?.error_for_status()?).await
        }
    }

    #[derive(Clone)]
    struct TestFileIO {}

    impl TestFileIO {
        fn init() -> Self {
            TestFileIO {}
        }
    }

    #[async_trait::async_trait]
    impl FileIO for TestFileIO {
        async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
            let mut file = tokio::fs::File::create(path).await?;
            file.write_all(content)
                .await
                .map_err(|e| anyhow!("{}", e))?;
            Ok(())
        }

        async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
            let mut file = tokio::fs::File::open(path).await?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .await
                .map_err(|e| anyhow!("{}", e))?;
            Ok(String::from_utf8(buffer)?)
        }
    }

    #[derive(Clone)]
    struct TestEnvIO {
        vars: HashMap<String, String>,
    }

    impl EnvIO for TestEnvIO {
        fn get(&self, key: &str) -> Option<String> {
            self.vars.get(key).cloned()
        }
    }

    impl TestEnvIO {
        pub fn init() -> Self {
            Self { vars: std::env::vars().collect() }
        }
    }

    pub fn init() -> TargetRuntime {
        let http: Arc<dyn HttpIO + Sync + Send> = Arc::new(TestHttp::init(false));

        let http2: Arc<dyn HttpIO + Sync + Send> = Arc::new(TestHttp::init(true));

        let file = TestFileIO::init();
        let env = TestEnvIO::init();

        TargetRuntime {
            http,
            http2_only: http2,
            env: Arc::new(env),
            file: Arc::new(file),
            cache: Arc::new(InMemoryCache::new()),
        }
    }
}
#[cfg(test)]
mod server_spec {
    use reqwest::Client;
    use serde_json::json;
    use tailcall::builder::TailcallBuilder;
    use tailcall::cli::server::Server;

    use crate::test;

    async fn test_server(configs: &[&str], url: &str) {
        let runtime = tailcall::cli::runtime::init(&Default::default(), None);
        let tailcall_executor = TailcallBuilder::init(runtime)
            .with_config_paths(configs)
            .build()
            .await
            .unwrap();
        let mut server = Server::new(tailcall_executor);
        let server_up_receiver = server.server_up_receiver();

        tokio::spawn(async move {
            server.fork_start().await.unwrap();
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
        let runtime = test::init();
        let tailcall_executor = TailcallBuilder::init(runtime).with_config_paths(configs);
        assert!(tailcall_executor.build().await.is_err())
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
