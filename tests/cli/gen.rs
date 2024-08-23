pub mod test {
    use std::path::Path;

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
        use tailcall::core::valid::{ValidateInto, Validator};
        use tokio::runtime::Runtime;

        use super::http::NativeHttpTest;

        pub fn run_config_generator_spec(path: &Path) -> datatest_stable::Result<()> {
            let path = path.to_path_buf();
            let runtime = Runtime::new().unwrap();
            runtime.block_on(async move {
                run_test(&path.to_string_lossy()).await?;
                Ok(())
            })
        }

        async fn run_test(path: &str) -> anyhow::Result<()> {
            let mut runtime = tailcall::cli::runtime::init(&Blueprint::default());
            runtime.http = Arc::new(NativeHttpTest::default());

            let generator = Generator::new(path, runtime);
            let config = generator.read().await?;
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

            insta::assert_snapshot!(path, config.to_sdl());
            Ok(())
        }
    }
    pub fn test_generator(path: &Path) -> datatest_stable::Result<()> {
        if let Some(extension) = path.extension() {
            if extension == "json"
                && path
                    .file_name()
                    .and_then(|v| v.to_str())
                    .map(|v| v.starts_with("gen"))
                    .unwrap_or_default()
            {
                let _ = generator_spec::run_config_generator_spec(path);
            }
        }
        Ok(())
    }
}

datatest_stable::harness!(
    test::test_generator,
    "tests/cli/fixtures/generator",
    r"^.*\.json"
);
