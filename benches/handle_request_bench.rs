use std::sync::Arc;

use criterion::Criterion;
use hyper::Request;
use serde_json::json;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::{Blueprint};
use tailcall::cli::server::server_config::ServerConfig;
use tailcall::config::{Config, ConfigModule};
use tailcall::http::{handle_request};
use tailcall::valid::Validator;

pub fn benchmark_handle_request(c: &mut Criterion) {
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let sdl = std::fs::read_to_string("./ci-benchmark/benchmark.graphql").unwrap();
    let config_module: ConfigModule = Config::from_sdl(sdl.as_str()).to_result().unwrap().into();

    let blueprint = Blueprint::try_from(&config_module).unwrap();
    let endpoints = config_module.extensions.endpoint_set;

    let server_config = tokio_runtime
        .block_on(ServerConfig::new(blueprint, endpoints))
        .unwrap();
    let server_config = Arc::new(server_config);

    c.bench_function("test_handle_request", |b| {
        let server_config = server_config.clone();
        b.iter(|| {
            let server_config = server_config.clone();
            tokio_runtime.spawn(async move {
                let req = Request::builder()
                    .method("POST")
                    .uri("http://localhost:8000/graphql")
                    .body(hyper::Body::from(
                        json!({
                            "query": "query { posts { title } }"
                        })
                        .to_string(),
                    ))
                    .unwrap();

                let _ = handle_request::<GraphQLRequest>(req, server_config.app_ctx.clone())
                    .await
                    .unwrap();
            });
        })
    });
}
