use std::sync::Arc;

use http::{to_request, to_response};
use hyper::service::Service;
use lambda_http::{run, service_fn, Body, Error, Response};
use runtime::init_runtime;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::reader::ConfigReader;
use tailcall::http::{create_request_service, AppContext};

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
    let config = ConfigReader::init(runtime.clone())
        .read("./config.graphql")
        .await?;
    let blueprint = Blueprint::try_from(&config)?;

    let app_ctx = Arc::new(AppContext::new(blueprint, runtime));

    run(service_fn(|req| async move {
        let resp = create_request_service::<GraphQLRequest>(app_ctx.clone(), "127.0.0.1".parse()?)
            .unwrap()
            .call(to_request(req)?)
            .await?;

        Ok::<_, Error>(to_response(resp).await?)
    }))
}
