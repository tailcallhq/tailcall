#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use hyper::server::conn::http2::Builder;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde::de::DeserializeOwned;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_rustls::TlsAcceptor;

use super::server_config::ServerConfig;
use crate::cli::CLIError;
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest, GraphQLRequestLike};
use crate::core::http::{handle_request, Request};

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
        .with_single_cert(cert, key.clone_key())
        .map_err(CLIError::from)?;

    tls_cfg.alpn_protocols = vec![
        b"h2".to_vec(),
        b"http/1.1".to_vec(),
        b"http/1.0".to_vec(),
        b"http/0.9".to_vec(),
    ];

    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_cfg));

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
        match stream_result {
            Ok((stream, _)) => {
                let app_ctx = sc.app_ctx.clone();
                let io_result = acceptor.accept(stream).await;
                match io_result {
                    Ok(io) => {
                        let io = TokioIo::new(io);
                        let server = builder
                            .serve_connection(
                                io,
                                service_fn(move |req| {
                                    let app_ctx = app_ctx.clone();
                                    async move {
                                        let req = Request::from_hyper(req).await?;
                                        handle_request::<T>(req, app_ctx).await
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
                    Err(e) => tracing::error!("An error occurred while handling request IO: {e}"),
                }
            }
            Err(e) => tracing::error!("An error occurred while handling request: {e}"),
        }
    }
}
