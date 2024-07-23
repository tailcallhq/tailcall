use std::sync::Arc;

use dotenvy::dotenv;
use http::{to_request, to_response};
use lambda_http::{run, service_fn, Body, Error, Response};
use runtime::init_runtime;
use tailcall::core::app_context::AppContext;
use tailcall::core::async_graphql_hyper::GraphQLRequest;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::config::reader::ConfigReader;
use tailcall::core::http::handle_request;
use tailcall::core::tracing::get_log_level;

mod http;
mod runtime;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = dotenv();

    let level: tracing::Level = get_log_level()
        // log everything by default since logs can be filtered by level in CloudWatch.
        .unwrap_or(tracing::Level::TRACE);

    tracing_subscriber::fmt()
        .with_max_level(level)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let runtime = init_runtime();
    let config = ConfigReader::init(runtime.clone())
        .read("./config.graphql")
        .await?;
    let blueprint = Blueprint::try_from(&config)?;
    let endpoints = config
        .extensions()
        .endpoint_set
        .clone()
        .into_checked(&blueprint, runtime.clone())
        .await?;

    let app_ctx = Arc::new(AppContext::new(blueprint, runtime, endpoints));

    run(service_fn(|event| async {
        let resp = handle_request::<GraphQLRequest>(to_request(event)?, app_ctx.clone()).await?;
        Ok::<Response<Body>, Error>(to_response(resp).await?)
    }))
    .await
}
