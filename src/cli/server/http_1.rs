use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use tokio::sync::oneshot;
use tower_http::cors::CorsLayer;

use super::server_config::ServerConfig;
use crate::async_graphql_hyper::GraphQLRequest;
use crate::cli::CLIError;
use crate::http::{handle_request, handle_request_with_cors};

pub async fn start_http_1(
    sc: Arc<ServerConfig>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let make_svc_single_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);

        async move {
            let state = state.clone();
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                let state = state.clone();
                async move {
                    let state = state.clone();
                    match state.app_ctx.blueprint.server.cors_params {
                        Some(ref cors_params) => {
                            handle_request_with_cors::<GraphQLRequest>(
                                req,
                                cors_params,
                                state.app_ctx.clone(),
                            )
                            .await
                        }
                        None => handle_request::<GraphQLRequest>(req, state.app_ctx.clone()).await,
                    }
                }
            }))
        }
    });

    let make_svc_batch_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        let _cors = CorsLayer::permissive();

        async move {
            let state = state.clone();
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                let state = state.clone();
                async move {
                    let state = state.clone();
                    match state.app_ctx.blueprint.server.cors_params {
                        Some(ref cors_params) => {
                            handle_request_with_cors::<GraphQLRequest>(
                                req,
                                cors_params,
                                state.app_ctx.clone(),
                            )
                            .await
                        }
                        None => handle_request::<GraphQLRequest>(req, state.app_ctx.clone()).await,
                    }
                }
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
