use std::sync::Arc;

use cache::LambdaCache;
use env::LambdaEnv;
use http::{to_request, to_response};
use lambda_http::{run, service_fn, Error, Request, Response};
use tailcall::{async_graphql_hyper::GraphQLRequest, blueprint::Blueprint, config::Config, http::{handle_request, AppContext}};

mod cache;
mod env;
mod http;

async fn function_handler(req: Request) -> Result<Response<hyper::Body>, Error> {
    let config = Config::default();
    let blueprint = Blueprint::try_from(&config)?;
    
    let h_client = Arc::new(http::LambdaHttp::init());
    let app_ctx = Arc::new(AppContext::new(
        blueprint,
        h_client.clone(),
        h_client,
        Arc::new(LambdaEnv),
        Arc::new(LambdaCache::new()),
        None,
    ));
    let resp = handle_request::<GraphQLRequest>(to_request(req), app_ctx).await?;

    Ok(to_response(resp)?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
