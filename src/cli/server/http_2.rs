#![allow(clippy::too_many_arguments)]
use std::sync::Arc;

use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper_rustls::TlsAcceptor;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::cli::CLIError;
use crate::http::{handle_request, handle_request_with_cors};

pub async fn start_http_2(
    sc: Arc<ServerConfig>,
    cert: Vec<CertificateDer<'static>>,
    key: Arc<PrivateKeyDer<'static>>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let incoming = AddrIncoming::bind(&addr)?;
    let acceptor = TlsAcceptor::builder()
        .with_single_cert(cert, key.clone_key())?
        .with_http2_alpn()
        .with_incoming(incoming);
    let make_svc_single_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                let state = state.clone();
                async move {
                    if let Some(ref cors_params) = state.blueprint.server.cors_params {
                        handle_request_with_cors::<GraphQLRequest>(
                            req,
                            cors_params,
                            state.app_ctx.clone(),
                        )
                        .await
                    } else {
                        handle_request::<GraphQLRequest>(req, state.app_ctx.clone()).await
                    }
                }
            }))
        }
    });

    let make_svc_batch_req = make_service_fn(|_conn| {
        let state = Arc::clone(&sc);
        async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                let state = state.clone();
                async move {
                    if let Some(ref cors_params) = state.blueprint.server.cors_params {
                        handle_request_with_cors::<GraphQLBatchRequest>(
                            req,
                            cors_params,
                            state.app_ctx.clone(),
                        )
                        .await
                    } else {
                        handle_request::<GraphQLBatchRequest>(req, state.app_ctx.clone()).await
                    }
                }
            }))
        }
    });

    let builder = Server::builder(acceptor).http2_only(true);

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
