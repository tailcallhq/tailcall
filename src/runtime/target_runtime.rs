use std::sync::Arc;

use async_graphql_value::ConstValue;

use crate::{Cache, EnvIO, FileIO, HttpIO};

/// The TargetRuntime struct unifies the available runtime-specific
/// IO implementations. This is used to reduce piping IO structs all
/// over the codebase.
#[derive(Clone)]
pub struct TargetRuntime {
    pub http: Arc<dyn HttpIO>,
    pub http2_only: Arc<dyn HttpIO>,
    pub env: Arc<dyn EnvIO>,
    pub file: Arc<dyn FileIO>,
    pub cache: Arc<dyn Cache<Key = u64, Value = ConstValue>>,
}

#[cfg(test)]
pub mod test_rt {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use anyhow::{anyhow, Result};
    use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
    use hyper::body::Bytes;
    use reqwest::Client;
    use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::blueprint::Upstream;
    use crate::cache::InMemoryCache;
    use crate::cli::javascript;
    use crate::http::Response;
    use crate::runtime::TargetRuntime;
    use crate::{blueprint, EnvIO, FileIO, HttpIO};

    #[derive(Clone)]
    pub struct TestHttp {
        client: ClientWithMiddleware,
    }

    impl Default for TestHttp {
        fn default() -> Self {
            Self { client: ClientBuilder::new(Client::new()).build() }
        }
    }

    impl TestHttp {
        pub fn init(upstream: &Upstream) -> Self {
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
                    manager: MokaManager::default(),
                    options: HttpCacheOptions::default(),
                }))
            }
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
    pub struct TestFileIO {}

    impl TestFileIO {
        pub fn init() -> Self {
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
    pub struct TestEnvIO {
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

    pub fn init_test_rt(script: Option<blueprint::Script>) -> TargetRuntime {
        let http: Arc<dyn HttpIO + Sync + Send> = if let Some(script) = script.clone() {
            javascript::init_http(TestHttp::init(&Default::default()), script)
        } else {
            Arc::new(TestHttp::init(&Default::default()))
        };

        let http2: Arc<dyn HttpIO + Sync + Send> = if let Some(script) = script {
            javascript::init_http(
                TestHttp::init(&Upstream::default().http2_only(true)),
                script,
            )
        } else {
            Arc::new(TestHttp::init(&Upstream::default().http2_only(true)))
        };

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
