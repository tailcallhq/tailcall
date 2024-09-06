#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use http_body_util::Full;
use hyper::server::conn::http2::Builder;
use hyper::service::service_fn;
use hyper::Response;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde::de::DeserializeOwned;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_rustls::TlsAcceptor;

use super::server_config::ServerConfig;
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest, GraphQLRequestLike};
use crate::core::http::handle_incoming;
use crate::core::Errata;

pub async fn start_http_2(
    sc: Arc<ServerConfig>,
    cert: Vec<CertificateDer<'static>>,
    key: Arc<PrivateKeyDer<'static>>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let mut tls_cfg = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key.clone_key())?;

    tls_cfg.alpn_protocols = vec![
        b"h2".to_vec(),
        b"http/1.1".to_vec(),
        b"http/1.0".to_vec(),
        b"http/0.9".to_vec(),
    ];

    let acceptor = TlsAcceptor::from(Arc::new(tls_cfg));

    let builder = hyper::server::conn::http2::Builder::new(TokioExecutor::new());

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }
    super::log_launch(sc.as_ref());

    if sc.blueprint.server.enable_batch_requests {
        handle::<GraphQLBatchRequest>(listener, sc, acceptor, builder).await
    } else {
        handle::<GraphQLRequest>(listener, sc, acceptor, builder).await
    }
}

async fn handle<T: DeserializeOwned + GraphQLRequestLike + Send>(
    listener: TcpListener,
    sc: Arc<ServerConfig>,
    acceptor: TlsAcceptor,
    builder: Builder<TokioExecutor>,
) -> anyhow::Result<()> {
    loop {
        let stream_result = listener.accept().await;
        if let Ok((stream, _)) = stream_result {
            let app_ctx = sc.app_ctx.clone();
            let io_result = acceptor.accept(stream).await;
            if let Ok(io) = io_result {
                let io = TokioIo::new(io);
                let server = builder
                    .serve_connection(
                        io,
                        service_fn(move |req| {
                            let app_ctx = app_ctx.clone();
                            async move {
                                handle_incoming::<T>(req, app_ctx)
                                    .await
                                    .map(|res| Response::new(Full::new(res.into_body())))
                                    .map_err(Errata::from)
                            }
                        }),
                    )
                    .await;
                tokio::spawn(async move {
                    if let Err(e) = server {
                        tracing::error!("An error occurred while handling a request: {e}");
                    }
                });
            }
        }
    }
}
