mod parser;

pub mod cacache_manager {
    use std::io::{Read, Write};
    use std::path::PathBuf;

    use flate2::write::GzEncoder;
    use flate2::Compression;
    use http_cache_reqwest::{CacheManager, HttpResponse};
    use http_cache_semantics::CachePolicy;
    use serde::{Deserialize, Serialize};

    pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
    pub type Result<T> = std::result::Result<T, BoxError>;

    pub struct CaCacheManager {
        path: PathBuf,
    }

    #[derive(Clone, Deserialize, Serialize)]
    pub struct Store {
        response: HttpResponse,
        policy: CachePolicy,
    }

    impl Default for CaCacheManager {
        fn default() -> Self {
            Self { path: PathBuf::from("./.cache") }
        }
    }

    #[async_trait::async_trait]
    impl CacheManager for CaCacheManager {
        async fn put(
            &self,
            cache_key: String,
            response: HttpResponse,
            policy: CachePolicy,
        ) -> Result<HttpResponse> {
            let data = Store { response: response.clone(), policy };
            let bytes = bincode::serialize(&data)?;

            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&bytes)?;
            let compressed_bytes = encoder.finish()?;

            cacache::write(&self.path, cache_key, compressed_bytes).await?;
            Ok(response)
        }

        async fn get(&self, cache_key: &str) -> Result<Option<(HttpResponse, CachePolicy)>> {
            match cacache::read(&self.path, cache_key).await {
                Ok(compressed_data) => {
                    let mut decoder = flate2::read::GzDecoder::new(compressed_data.as_slice());
                    let mut serialized_data = Vec::new();
                    decoder.read_to_end(&mut serialized_data)?;
                    let store: Store = bincode::deserialize(&serialized_data)?;
                    Ok(Some((store.response, store.policy)))
                }
                Err(_) => Ok(None),
            }
        }

        async fn delete(&self, cache_key: &str) -> Result<()> {
            Ok(cacache::remove(&self.path, cache_key).await?)
        }
    }
}

pub mod file {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use tailcall::core::FileIO;
    use tokio::sync::RwLock;

    #[derive(Clone, Default)]
    pub struct NativeFileTest(Arc<RwLock<HashMap<String, String>>>);
    #[async_trait]
    impl FileIO for NativeFileTest {
        async fn write<'a>(&'a self, path: &'a str, content: &'a [u8]) -> anyhow::Result<()> {
            self.0.write().await.insert(
                path.to_string(),
                String::from_utf8_lossy(content).to_string(),
            );
            Ok(())
        }

        async fn read<'a>(&'a self, path: &'a str) -> anyhow::Result<String> {
            let val = if let Some(val) = self.0.read().await.get(path).cloned() {
                val
            } else {
                std::fs::read_to_string(path)?
            };
            Ok(val)
        }
    }
}

pub mod http {
    use anyhow::Result;
    use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
    use hyper::body::Bytes;
    use reqwest::Client;
    use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
    use tailcall::core::http::Response;
    use tailcall::core::HttpIO;

    use super::cacache_manager::CaCacheManager;

    #[derive(Clone)]
    pub struct NativeHttpTest {
        client: ClientWithMiddleware,
    }

    impl Default for NativeHttpTest {
        fn default() -> Self {
            let mut client = ClientBuilder::new(Client::new());
            client = client.with(Cache(HttpCache {
                mode: CacheMode::ForceCache,
                manager: CaCacheManager::default(),
                options: HttpCacheOptions::default(),
            }));
            Self { client: client.build() }
        }
    }

    #[async_trait::async_trait]
    impl HttpIO for NativeHttpTest {
        #[allow(clippy::blocks_in_conditions)]
        async fn execute(&self, request: reqwest::Request) -> Result<Response<Bytes>> {
            let response = self.client.execute(request).await;
            Ok(Response::from_reqwest(
                response?
                    .error_for_status()
                    .map_err(|err| err.without_url())?,
            )
            .await?)
        }
    }
}
pub mod env {
    use std::borrow::Cow;
    use std::collections::HashMap;

    use tailcall::core::EnvIO;

    #[derive(Clone)]
    pub struct Env(pub HashMap<String, String>);

    impl EnvIO for Env {
        fn get(&self, key: &str) -> Option<Cow<'_, str>> {
            self.0.get(key).map(Cow::from)
        }
    }
}

pub mod test {
    use std::path::Path;

    use crate::parser::ExecutionSpec;

    mod cacache_manager {
        use std::io::{Read, Write};
        use std::path::PathBuf;

        use flate2::write::GzEncoder;
        use flate2::Compression;
        use http_cache_reqwest::{CacheManager, HttpResponse};
        use http_cache_semantics::CachePolicy;
        use serde::{Deserialize, Serialize};

        pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
        pub type Result<T> = std::result::Result<T, BoxError>;

        pub struct CaCacheManager {
            path: PathBuf,
        }

        #[derive(Clone, Deserialize, Serialize)]
        pub struct Store {
            response: HttpResponse,
            policy: CachePolicy,
        }

        impl Default for CaCacheManager {
            fn default() -> Self {
                Self { path: PathBuf::from("./.cache") }
            }
        }

        #[async_trait::async_trait]
        impl CacheManager for CaCacheManager {
            async fn put(
                &self,
                cache_key: String,
                response: HttpResponse,
                policy: CachePolicy,
            ) -> Result<HttpResponse> {
                let data = Store { response: response.clone(), policy };
                let bytes = bincode::serialize(&data)?;

                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&bytes)?;
                let compressed_bytes = encoder.finish()?;

                cacache::write(&self.path, cache_key, compressed_bytes).await?;
                Ok(response)
            }

            async fn get(&self, cache_key: &str) -> Result<Option<(HttpResponse, CachePolicy)>> {
                match cacache::read(&self.path, cache_key).await {
                    Ok(compressed_data) => {
                        let mut decoder = flate2::read::GzDecoder::new(compressed_data.as_slice());
                        let mut serialized_data = Vec::new();
                        decoder.read_to_end(&mut serialized_data)?;
                        let store: Store = bincode::deserialize(&serialized_data)?;
                        Ok(Some((store.response, store.policy)))
                    }
                    Err(_) => Ok(None),
                }
            }

            async fn delete(&self, cache_key: &str) -> Result<()> {
                Ok(cacache::remove(&self.path, cache_key).await?)
            }
        }
    }

    mod http {
        use anyhow::Result;
        use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
        use hyper::body::Bytes;
        use reqwest::Client;
        use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
        use tailcall::core::http::Response;
        use tailcall::core::HttpIO;

        use super::cacache_manager::CaCacheManager;

        #[derive(Clone)]
        pub struct NativeHttpTest {
            client: ClientWithMiddleware,
        }

        impl Default for NativeHttpTest {
            fn default() -> Self {
                let mut client = ClientBuilder::new(Client::new());
                client = client.with(Cache(HttpCache {
                    mode: CacheMode::ForceCache,
                    manager: CaCacheManager::default(),
                    options: HttpCacheOptions::default(),
                }));
                Self { client: client.build() }
            }
        }

        #[async_trait::async_trait]
        impl HttpIO for NativeHttpTest {
            #[allow(clippy::blocks_in_conditions)]
            async fn execute(&self, request: reqwest::Request) -> Result<Response<Bytes>> {
                let response = self.client.execute(request).await;
                Ok(Response::from_reqwest(
                    response?
                        .error_for_status()
                        .map_err(|err| err.without_url())?,
                )
                .await?)
            }
        }
    }

    mod generator_spec {
        use std::path::Path;
        use std::sync::Arc;

        use tailcall::cli::generator::Generator;
        use tailcall::core::blueprint::Blueprint;
        use tailcall::core::config::{self, ConfigModule};
        use tailcall::core::generator::Generator as ConfigGenerator;
        use tailcall_valid::{ValidateInto, Validator};

        use super::http::NativeHttpTest;
        use crate::env::Env;
        use crate::parser::{ExecutionSpec, IO};

        pub async fn run_test(original_path: &Path, spec: ExecutionSpec) -> anyhow::Result<()> {
            let snapshot_name = original_path.to_string_lossy().to_string();

            let IO { fs, paths } = spec.configs.into_io().await;
            let path = paths.first().unwrap().as_str();

            let mut runtime = tailcall::cli::runtime::init(&Blueprint::default());
            runtime.http = Arc::new(NativeHttpTest::default());
            runtime.file = Arc::new(fs);
            if let Some(env) = spec.env {
                runtime.env = Arc::new(Env(env))
            }

            let generator = Generator::new(path, runtime);
            let config = generator.read().await?;
            if spec.debug_assert_config {
                insta::assert_debug_snapshot!(snapshot_name, config);
                return Ok(());
            }

            let query_type = config.schema.query.clone().unwrap_or("Query".into());
            let mutation_type_name = config.schema.mutation.clone();
            let preset: config::transformer::Preset = config
                .preset
                .clone()
                .unwrap_or_default()
                .validate_into()
                .to_result()?;

            // resolve i/o's
            let input_samples = generator.resolve_io(config).await?;

            let cfg_module = ConfigGenerator::default()
                .query(query_type)
                .mutation(mutation_type_name)
                .inputs(input_samples)
                .transformers(vec![Box::new(preset)])
                .generate(true)?;

            // remove links since they break snapshot tests
            let mut base_config = cfg_module.config().clone();
            base_config.links = Default::default();

            let config = ConfigModule::from(base_config);

            insta::assert_snapshot!(snapshot_name, config.to_sdl());
            Ok(())
        }
    }
    async fn test_generator(path: &Path) -> datatest_stable::Result<()> {
        let spec = ExecutionSpec::from_source(path, std::fs::read_to_string(path)?)?;
        generator_spec::run_test(path, spec).await?;
        Ok(())
    }
    pub fn run(path: &Path) -> datatest_stable::Result<()> {
        tokio_test::block_on(test_generator(path))
    }
}

datatest_stable::harness!(test::run, "tests/cli/fixtures/generator", r"^.*\.md");
