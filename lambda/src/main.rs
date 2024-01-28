use std::sync::Arc;

use cache::LambdaCache;
use env::LambdaEnv;
use file::init_file;
use http::{init_http, to_request, to_response};
use lambda_http::{run, service_fn, Error, Request, Response};
use lazy_static::lazy_static;
use tailcall::async_graphql_hyper::GraphQLRequest;
use tailcall::blueprint::Blueprint;
use tailcall::config::reader::ConfigReader;
use tailcall::http::{handle_request, AppContext};
use tokio::sync::RwLock;

mod cache;
mod env;
mod file;
mod http;

lazy_static! {
    static ref APP_CTX: RwLock<Option<Arc<AppContext>>> = RwLock::new(None);
}

async fn function_handler(event: Request) -> Result<Response<hyper::Body>, Error> {
    let app_ctx = APP_CTX
        .read()
        .await
        .clone()
        .expect("AppContext not initialized yet, please wait");
    let resp = handle_request::<GraphQLRequest>(to_request(event), app_ctx).await?;
    Ok(to_response(resp)?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let file = init_file();
    let http = init_http();

    let config = ConfigReader::init(file, http.clone())
        .read("./config.graphql")
        .await?;
    let blueprint = Blueprint::try_from(&config)?;

    let app_ctx = Arc::new(AppContext::new(
        blueprint,
        http.clone(),
        http,
        Arc::new(LambdaEnv),
        Arc::new(LambdaCache::new()),
    ));

    APP_CTX.write().await.replace(app_ctx);

    run(service_fn(function_handler)).await
}
