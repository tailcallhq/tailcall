use std::sync::Arc;

use criterion::Criterion;
use hyper::Request;
use tailcall::cli::server::server_config::ServerConfig;
use tailcall::core::async_graphql_hyper::GraphQLRequest;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::config::{Config, ConfigModule};
use tailcall::core::http::handle_request;
use tailcall::core::valid::Validator;

static QUERY: &str = r#"{"query":"query{posts{title}}"}"#;

pub fn benchmark_handle_request(c: &mut Criterion) {
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let sdl = std::fs::read_to_string("./ci-benchmark/benchmark.graphql").unwrap();
    let config_module: ConfigModule = Config::from_sdl(sdl.as_str()).to_result().unwrap().into();

    let blueprint = Blueprint::try_from(&config_module).unwrap();
    let endpoints = config_module.extensions().endpoint_set.clone();

    let server_config = tokio_runtime
        .block_on(ServerConfig::new(blueprint, endpoints))
        .unwrap();
    let server_config = Arc::new(server_config);

    c.bench_function("test_handle_request", |b| {
        let server_config = server_config.clone();
        b.iter(|| {
            let server_config = server_config.clone();
            tokio_runtime.block_on(async move {
                let req = Request::builder()
                    .method("POST")
                    .uri("http://localhost:8000/graphql")
                    .body(hyper::Body::from(QUERY))
                    .unwrap();

                let _ = handle_request::<GraphQLRequest>(req, server_config.app_ctx.clone())
                    .await
                    .unwrap();
            });
        })
    });
}
