use std::sync::Arc;

use http_body_util::Full;
use hyper::server::conn::http1::Builder;
use hyper::service::service_fn;
use hyper::Response;
use hyper_util::rt::TokioIo;
use serde::de::DeserializeOwned;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest, GraphQLRequestLike};
use crate::core::http::handle_incoming;
use crate::core::Errata;

pub async fn start_http_1(
    sc: Arc<ServerConfig>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let listener = TcpListener::bind(&addr).await?;
    let mut builder = Builder::new();

    builder.keep_alive(true);
    super::log_launch(sc.as_ref());

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }
    if sc.blueprint.server.enable_batch_requests {
        handle::<GraphQLBatchRequest>(listener, sc, builder).await
    } else {
        handle::<GraphQLRequest>(listener, sc, builder).await
    }
}

async fn handle<T: DeserializeOwned + GraphQLRequestLike + Send>(
    listener: TcpListener,
    sc: Arc<ServerConfig>,
    builder: Builder,
) -> anyhow::Result<()> {
    loop {
        let stream = listener.accept().await;
        let app_ctx = sc.app_ctx.clone();
        if let Ok((stream, _)) = stream {
            let connection = builder.serve_connection(
                TokioIo::new(stream),
                service_fn(move |req| {
                    let app_ctx = app_ctx.clone();
                    async move {
                        handle_incoming::<T>(req, app_ctx)
                            .await
                            .map(|res| Response::new(Full::new(res.into_body())))
                            .map_err(Errata::from)
                    }
                }),
            );
            tokio::spawn(async move {
                if let Err(err) = connection.await {
                    tracing::error!("Error serving HTTP connection: {err:?}");
                }
            });
        }
    }
}
