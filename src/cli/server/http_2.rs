#![allow(clippy::too_many_arguments)]
use std::sync::Arc;

use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde::de::DeserializeOwned;
use tokio::net::TcpListener;
use tokio::sync::oneshot::Sender;

use super::server_config::ServerConfig;
use crate::cli::server::log_launch;
use crate::cli::CLIError;
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest, GraphQLRequestLike};
use crate::core::http::{handle_request, RequestBody};

pub async fn start_http_2(
    sc: Arc<ServerConfig>,
    cert: Vec<CertificateDer<'static>>,
    key: Arc<PrivateKeyDer<'static>>,
    server_up_sender: Option<Sender<()>>,
) -> anyhow::Result<()> {
    if sc.blueprint.server.enable_batch_requests {
        start::<GraphQLBatchRequest>(sc, cert, key, server_up_sender).await
    } else {
        start::<GraphQLRequest>(sc, cert, key, server_up_sender).await
    }
}

async fn start<T: DeserializeOwned + GraphQLRequestLike + Send>(
    sc: Arc<ServerConfig>,
    cert: Vec<CertificateDer<'static>>,
    key: Arc<PrivateKeyDer<'static>>,
    server_up_sender: Option<Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let listener = TcpListener::bind(addr).await?;
    let mut server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key.clone_key())
        .map_err(CLIError::from)?;

    server_config.alpn_protocols = vec![
        b"h2".to_vec(),
        b"http/1.1".to_vec(),
        b"http/1.0".to_vec(),
        b"http/0.9".to_vec(),
    ];

    let tls_acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }

    log_launch(sc.as_ref());
    loop {
        let stream_result = listener.accept().await;
        match stream_result {
            Ok((stream, _)) => {
                let app_ctx = sc.app_ctx.clone();
                let io_result = tls_acceptor.accept(stream).await;
                match io_result {
                    Ok(io) => {
                        let io = TokioIo::new(io);
                        tokio::spawn(async move {
                            let server =
                                hyper::server::conn::http2::Builder::new(TokioExecutor::new())
                                    .serve_connection(
                                        io,
                                        service_fn(move |req: Request<Incoming>| {
                                            let app_ctx = app_ctx.clone();
                                            async move {
                                                let (parts, body) = req.into_parts();
                                                let req = Request::from_parts(
                                                    parts,
                                                    RequestBody::Incoming(body),
                                                );
                                                handle_request::<T>(req, app_ctx).await
                                            }
                                        }),
                                    )
                                    .await;
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
