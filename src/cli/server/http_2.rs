#![allow(clippy::too_many_arguments)]
use std::sync::Arc;

use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper_rustls::TlsAcceptor;
use rustls_pki_types::CertificateDer;
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::core::config::PrivateKey;
use crate::core::http::handle_request;
use crate::core::Errata;

pub async fn start_http_2(
    sc: Arc<ServerConfig>,
    cert: Vec<CertificateDer<'static>>,
    key: PrivateKey,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let incoming = AddrIncoming::bind(&addr)?;
    let acceptor = TlsAcceptor::builder()
        .with_single_cert(cert, key.into_inner())?
        .with_http2_alpn()
        .with_incoming(incoming);
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

    let builder = Server::builder(acceptor).http2_only(true);

    super::log_launch(sc.as_ref());

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

    let result = server.map_err(Errata::from);

    Ok(result?)
}
