use std::sync::Arc;

use hyper::service::make_service_fn;
use tokio::sync::oneshot;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use super::server_config::ServerConfig;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::cli::CLIError;
use crate::http::handle_request;

pub async fn start_http_1(
    sc: Arc<ServerConfig>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let make_svc_single_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        let cors = CorsLayer::permissive();

        async move {
            Ok::<_, anyhow::Error>(ServiceBuilder::new().layer(cors).service_fn(move |req| {
                handle_request::<GraphQLRequest>(req, state.app_ctx.clone())
            }))
        }
    });

    let make_svc_batch_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        let cors = CorsLayer::permissive();

        async move {
            Ok::<_, anyhow::Error>(ServiceBuilder::new().layer(cors).service_fn(move |req| {
                handle_request::<GraphQLBatchRequest>(req, state.app_ctx.clone())
            }))
        }
    });
    let builder = hyper::Server::try_bind(&addr)
        .map_err(CLIError::from)?
        .http1_pipeline_flush(sc.app_ctx.blueprint.server.pipeline_flush);
    super::log_launch_and_open_browser(sc.as_ref());

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }

    let server: std::prelude::v1::Result<(), hyper::Error> =
        if sc.blueprint.server.enable_batch_requests {
            builder.serve(make_svc_batch_req).await
        } else {
            builder.serve(make_svc_single_req).await
        };

    let result = server.map_err(CLIError::from);

    Ok(result?)
}
