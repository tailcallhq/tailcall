#![allow(clippy::too_many_arguments)]
use std::sync::Arc;

use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use hyper_rustls::TlsAcceptor;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::cli::CLIError;

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
    let make_svc_req = make_service_fn(|_conn| {
        let state = sc.tailcall_executor.clone();
        async move { Ok::<_, anyhow::Error>(service_fn(move |req| state.clone().execute(req))) }
    });

    let builder = Server::builder(acceptor).http2_only(true);

    super::log_launch_and_open_browser(sc.as_ref());

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }

    let server: std::prelude::v1::Result<(), hyper::Error> = builder.serve(make_svc_req).await;

    let result = server.map_err(CLIError::from);

    Ok(result?)
}
