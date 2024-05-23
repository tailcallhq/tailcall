#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::cli::CLIError;
use crate::core::async_graphql_hyper::GraphQLRequest;
use crate::core::http::{handle_request, Request};

pub async fn start_http_2(
    sc: Arc<ServerConfig>,
    cert: Vec<CertificateDer<'static>>,
    key: Arc<PrivateKeyDer<'static>>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let tls_cfg = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key.clone_key())
        .map_err(CLIError::from)?;

    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_cfg));

    let builder = hyper::server::conn::http2::Builder::new(TokioExecutor::new());

    /*let mut _ty: impl GraphQLRequestLike + DeserializeOwned = GraphQLRequest;

    if sc.blueprint.server.enable_batch_requests {
        _ty = GraphQLBatchRequest;
    };*/

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }
    super::log_launch(sc.as_ref());

    loop {
        let (stream, _) = listener.accept().await?;
        let stream = acceptor.accept(stream).await?;
        let app_ctx = sc.app_ctx.clone();

        let connection = builder.serve_connection(
            TokioIo::new(stream),
            service_fn(move |req| {
                let app_ctx = app_ctx.clone();
                async move {
                    let req = Request::from_hyper(req).await?;
                    handle_request::<
                        GraphQLRequest, // TODO
                    >(req, app_ctx)
                    .await
                }
            }),
        );
        tokio::spawn(async move {
            if let Err(err) = connection.await {
                println!("Error serving HTTP connection: {err:?}");
            }
        });
    }

    /*    let builder = Server::builder(acceptor).http2_only(true);

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

    Ok(result?)*/
}
