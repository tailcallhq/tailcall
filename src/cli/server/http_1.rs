use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn, Service};
use routerify::RouteError;
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::cli::CLIError;
use crate::http::create_request_service;

pub async fn start_http_1(
    sc: Arc<ServerConfig>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();

    let make_svc_single_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        async move {
            let mut service =
                create_request_service::<GraphQLRequest>(state.app_ctx.clone(), addr)?;
            Ok::<_, RouteError>(service_fn(move |req| service.call(req)))
        }
    });

    let make_svc_batch_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        async move {
            let mut service =
                create_request_service::<GraphQLBatchRequest>(state.app_ctx.clone(), addr)?;
            Ok::<_, RouteError>(service_fn(move |req| service.call(req)))
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

    let server: Result<(), hyper::Error> = if sc.blueprint.server.enable_batch_requests {
        builder.serve(make_svc_batch_req).await
    } else {
        builder.serve(make_svc_single_req).await
    };

    let result = server.map_err(CLIError::from);

    Ok(result?)
}
