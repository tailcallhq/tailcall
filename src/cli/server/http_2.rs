#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde::de::DeserializeOwned;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;

use super::server_config::ServerConfig;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest, GraphQLRequestLike};
use crate::cli::CLIError;
use crate::http::handle_request;

pub async fn start_http_2(
    sc: Arc<ServerConfig>,
    cert: Vec<CertificateDer<'static>>,
    key: Arc<PrivateKeyDer<'static>>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    super::log_launch_and_open_browser(sc.as_ref());

    if sc.blueprint.server.enable_batch_requests {
        run::<GraphQLBatchRequest>(sc, cert, key, server_up_sender).await
    } else {
        run::<GraphQLRequest>(sc, cert, key, server_up_sender).await
    }
}

async fn run<T: DeserializeOwned + GraphQLRequestLike + Send>(
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
                                                let (part, body) = req.into_parts();
                                                let body = body.collect().await?.to_bytes();
                                                let req =
                                                    Request::from_parts(part, Full::new(body));
                                                handle_request::<T>(req, app_ctx).await
                                            }
                                        }),
                                    )
                                    .await;
                            if let Err(e) = server {
                                log::error!("An error occurred while handling a request: {e}");
                            }
                        });
                    }
                    Err(e) => log::error!("An error occurred while handling request IO: {e}"),
                }
            }
            Err(e) => log::error!("An error occurred while handling request: {e}"),
        }
    }
}
