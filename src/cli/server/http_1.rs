use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::cli::CLIError;
#[cfg(not(feature = "jit"))]
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::core::http::handle_request;
#[cfg(feature = "jit")]
use crate::core::jit::Request;

pub async fn start_http_1(
    sc: Arc<ServerConfig>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();

    #[cfg(feature = "jit")]
    let jit_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                handle_request::<Request<async_graphql::Value>>(req, state.app_ctx.clone())
            }))
        }
    });

    #[cfg(not(feature = "jit"))]
    let _make_svc_single_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                handle_request::<GraphQLRequest>(req, state.app_ctx.clone())
            }))
        }
    });

    #[cfg(not(feature = "jit"))]
    let _make_svc_batch_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                handle_request::<GraphQLBatchRequest>(req, state.app_ctx.clone())
            }))
        }
    });
    let builder = hyper::Server::try_bind(&addr)
        .map_err(CLIError::from)?
        .http1_pipeline_flush(sc.app_ctx.blueprint.server.pipeline_flush);
    super::log_launch(sc.as_ref());

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }

    #[cfg(not(feature = "jit"))]
    let server: std::prelude::v1::Result<(), hyper::Error> =
        if sc.blueprint.server.enable_batch_requests {
            builder.serve(_make_svc_batch_req).await
        } else {
            builder.serve(_make_svc_single_req).await
        };

    #[cfg(feature = "jit")]
    let server = builder.serve(jit_req).await;

    let result = server.map_err(CLIError::from);

    Ok(result?)
}
