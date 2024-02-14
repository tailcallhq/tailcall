use http::{to_request, to_response};
use lambda_http::{run, service_fn, Body, Error, Response};
use runtime::init_runtime;
use tailcall::builder::TailcallBuilder;

mod http;
mod runtime;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let runtime = init_runtime();
    let tailcall_executor = TailcallBuilder::init(runtime)
        .with_config_paths(&["./config.graphql"])
        .await?;
    run(service_fn(|event| async {
        let resp = tailcall_executor.execute(to_request(event)?).await?;
        Ok::<Response<Body>, Error>(to_response(resp).await?)
    }))
    .await
}
