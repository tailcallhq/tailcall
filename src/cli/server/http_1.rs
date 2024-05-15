use std::sync::Arc;

use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::{TokioIo, TokioTimer};
use serde::de::DeserializeOwned;
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::cli::server::log_launch;
use crate::cli::CLIError;
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest, GraphQLRequestLike};
use crate::core::http::{handle_request, RequestBody};

pub async fn start_http_1(
    sc: Arc<ServerConfig>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    if sc.blueprint.server.enable_batch_requests {
        start::<GraphQLBatchRequest>(sc, server_up_sender).await
    } else {
        start::<GraphQLRequest>(sc, server_up_sender).await
    }
}

async fn start<T: DeserializeOwned + GraphQLRequestLike + Send>(
    sc: Arc<ServerConfig>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let listener = std::net::TcpListener::bind(addr).map_err(CLIError::from)?;

    listener.set_nonblocking(true)?;

    let listener = tokio::net::TcpListener::from_std(listener).map_err(CLIError::from)?;

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }

    log_launch(sc.as_ref());
    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);
        let sc = sc.clone();
        tokio::task::spawn(async move {
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .timer(TokioTimer::new())
                .pipeline_flush(true)
                .serve_connection(
                    io,
                    service_fn(move |req: Request<Incoming>| {
                        let state = sc.clone();
                        async move {
                            let (parts, body) = req.into_parts();
                            let req = Request::from_parts(parts, RequestBody::Incoming(body));
                            handle_request::<T>(req, state.app_ctx.clone()).await
                        }
                    }),
                )
                .await
            {
                tracing::error!("An error occurred while handling a request 1: {err}");
            }
        });
    }
}
