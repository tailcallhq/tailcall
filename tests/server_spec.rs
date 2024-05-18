#[cfg(test)]
pub mod test {
    use std::borrow::Cow;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use anyhow::{anyhow, Result};
    use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
    use hyper::body::Bytes;
    use reqwest::Client;
    use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
    use tailcall::cli::javascript;
    use tailcall::core::blueprint::{Script, Upstream};
    use tailcall::core::cache::InMemoryCache;
    use tailcall::core::http::Response;
    use tailcall::core::runtime::TargetRuntime;
    use tailcall::core::{EnvIO, FileIO, HttpIO};
    use tailcall_http_cache::HttpCacheManager;
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
        fn init(upstream: &Upstream) -> Arc<Self> {
            let mut builder = Client::builder()
                .tcp_keepalive(Some(Duration::from_secs(upstream.tcp_keep_alive)))
                .timeout(Duration::from_secs(upstream.timeout))
                .connect_timeout(Duration::from_secs(upstream.connect_timeout))
                .http2_keep_alive_interval(Some(Duration::from_secs(upstream.keep_alive_interval)))
                .http2_keep_alive_timeout(Duration::from_secs(upstream.keep_alive_timeout))
                .http2_keep_alive_while_idle(upstream.keep_alive_while_idle)
                .pool_idle_timeout(Some(Duration::from_secs(upstream.pool_idle_timeout)))
                .pool_max_idle_per_host(upstream.pool_max_idle_per_host)
                .user_agent(upstream.user_agent.clone());

            // Add Http2 Prior Knowledge
            if upstream.http2_only {
                builder = builder.http2_prior_knowledge();
            }

            // Add Http Proxy
            if let Some(ref proxy) = upstream.proxy {
                builder = builder.proxy(
                    reqwest::Proxy::http(proxy.url.clone())
                        .expect("Failed to set proxy in http client"),
                );
            }

            let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));

            if upstream.http_cache {
                client = client.with(Cache(HttpCache {
                    mode: CacheMode::Default,
                    manager: HttpCacheManager::default(),
                    options: HttpCacheOptions::default(),
                }))
            }
            Arc::new(Self { client: client.build() })
        }
    }

    #[async_trait::async_trait]
    impl HttpIO for TestHttp {
        async fn execute(&self, request: reqwest::Request) -> Result<Response<Bytes>> {
            let response = self.client.execute(request).await;
            Response::from_reqwest(
                response?
                    .error_for_status()
                    .map_err(|err| err.without_url())?,
            )
            .await
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
        fn get(&self, key: &str) -> Option<Cow<'_, str>> {
            self.vars.get(key).map(Cow::from)
        }
    }

    impl TestEnvIO {
        pub fn init() -> Self {
            Self { vars: std::env::vars().collect() }
        }
    }

    pub fn init(script: Option<Script>) -> TargetRuntime {
        let http = if let Some(script) = script.clone() {
            javascript::init_http(TestHttp::init(&Default::default()), script)
        } else {
            TestHttp::init(&Default::default())
        };

        let http2 = if let Some(script) = script {
            javascript::init_http(
                TestHttp::init(&Upstream::default().http2_only(true)),
                script,
            )
        } else {
            TestHttp::init(&Upstream::default().http2_only(true))
        };

        let file = TestFileIO::init();
        let env = TestEnvIO::init();

        TargetRuntime {
            http,
            http2_only: http2,
            env: Arc::new(env),
            file: Arc::new(file),
            cache: Arc::new(InMemoryCache::new()),
            extensions: Arc::new(vec![]),
        }
    }
}

#[cfg(test)]
mod server_spec {
    use reqwest::Client;
    use serde_json::json;
    use tailcall::cli::server::Server;
    use tailcall::core::config::reader::ConfigReader;

    async fn test_server(configs: &[&str], url: &str) {
        let runtime = crate::test::init(None);
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
            .use_rustls_tls()
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
        let runtime = crate::test::init(None);
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
