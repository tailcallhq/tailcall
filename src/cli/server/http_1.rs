use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::core::http::handle_request;
use crate::core::Errata;

pub async fn start_http_1(
    sc: Arc<ServerConfig>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let make_svc_single_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                handle_request::<GraphQLRequest>(req, state.app_ctx.clone())
            }))
        }
    });

    let make_svc_batch_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                handle_request::<GraphQLBatchRequest>(req, state.app_ctx.clone())
            }))
        }
    });
    let builder = hyper::Server::try_bind(&addr)
        .map_err(Errata::from)?
        .http1_pipeline_flush(sc.app_ctx.blueprint.server.pipeline_flush);
    super::log_launch(sc.as_ref());

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }

    let proxy_server = tokio::spawn(async {
        let proxy_service = make_service_fn(|_conn| async {
            Ok::<_, anyhow::Error>(service_fn(super::http_proxy::handle))
        });

        let addr = SocketAddr::from_str("127.0.0.1:8100").unwrap();
        let builder = hyper::Server::try_bind(&addr)?;

        builder.serve(proxy_service).await
    });

    let server = async {
        if sc.blueprint.server.enable_batch_requests {
            builder.serve(make_svc_batch_req).await
        } else {
            builder.serve(make_svc_single_req).await
        }
    };

    let (server_result, proxy_server_result) = tokio::join!(server, proxy_server);

    proxy_server_result?.map_err(Errata::from)?;
    let server_result = server_result.map_err(Errata::from);

    Ok(server_result?)
}
