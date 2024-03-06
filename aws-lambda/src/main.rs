use std::str::FromStr as _;
use dotenvy::dotenv;

use http::{to_request, to_response};
use lambda_http::{run, service_fn, Body, Error, Response};
use runtime::init_runtime;
use tailcall::TailcallBuilder;

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
    let tailcall_executor = TailcallBuilder::new()
        .with_config_files(&["./config.graphql"])
        .build(runtime)
        .await?;
    run(service_fn(|event| async {
        let resp = tailcall_executor
            .clone()
            .execute(to_request(event)?)
            .await?;
        Ok::<Response<Body>, Error>(to_response(resp).await?)
    }))
    .await
}
