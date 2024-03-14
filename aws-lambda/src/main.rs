use std::str::FromStr as _;
use std::sync::Arc;

use dotenvy::dotenv;
use http::{to_request, to_response};
use lambda_http::{run, service_fn, Body, Error, Response};
use runtime::init_runtime;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::reader::ConfigReader;
use tailcall::http::{handle_request, AppContext};

mod http;
mod runtime;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = dotenv();

    let trace: tracing::Level = std::env::var("TC_LOG_LEVEL")
        .ok()
        .or_else(|| std::env::var("TAILCALL_LOG_LEVEL").ok())
        .as_ref()
        .and_then(|x| tracing::Level::from_str(x).ok())
        // log everything by default since logs can be filtered by level in CloudWatch.
        .unwrap_or(tracing::Level::TRACE);

    tracing_subscriber::fmt()
        .with_max_level(trace)
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

    let app_ctx = Arc::new(AppContext::new(blueprint, runtime, config.extensions.endpoints).await?);

    run(service_fn(|event| async {
        let resp = handle_request::<GraphQLRequest>(to_request(event)?, app_ctx.clone()).await?;
        Ok::<Response<Body>, Error>(to_response(resp).await?)
    }))
    .await
}
