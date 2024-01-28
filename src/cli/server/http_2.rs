#![allow(clippy::too_many_arguments)]
use std::io::BufReader;
use std::sync::Arc;

use anyhow::Result;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::TokioIo;
use rustls_pki_types::{
    CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer, PrivatePkcs8KeyDer, PrivateSec1KeyDer,
};
use tokio::fs::File;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::cli::CLIError;
use crate::http::handle_request;

async fn load_cert(filename: String) -> Result<Vec<CertificateDer<'static>>, std::io::Error> {
    let file = File::open(filename).await?;
    let file = file.into_std().await;
    let mut file = BufReader::new(file);

    let certificates = rustls_pemfile::certs(&mut file)?;

    Ok(certificates.into_iter().map(CertificateDer::from).collect())
}

async fn load_private_key(filename: String) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(filename).await?;
    let file = file.into_std().await;
    let mut file = BufReader::new(file);

    let keys = rustls_pemfile::read_all(&mut file)?;

    if keys.len() != 1 {
        return Err(CLIError::new("Expected a single private key").into());
    }

    let key = keys.into_iter().find_map(|key| match key {
        rustls_pemfile::Item::RSAKey(key) => {
            Some(PrivateKeyDer::Pkcs1(PrivatePkcs1KeyDer::from(key)))
        }
        rustls_pemfile::Item::ECKey(key) => Some(PrivateKeyDer::Sec1(PrivateSec1KeyDer::from(key))),
        rustls_pemfile::Item::PKCS8Key(key) => {
            Some(PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key)))
        }
        _ => None,
    });

    key.ok_or(CLIError::new("Invalid private key").into())
}

#[derive(Clone, Copy, Debug)]
struct LocalExec;

impl<F> hyper::rt::Executor<F> for LocalExec
where
    F: std::future::Future + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn_local(fut);
    }
}

pub async fn start_http_2(
    sc: Arc<ServerConfig>,
    cert: Option<String>,
    key: Option<String>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> Result<()> {
    let addr = sc.addr();
    let listener = TcpListener::bind(addr).await?;

    if cert.is_some() && key.is_some() {
        // TODO add support for tls
        let _ = load_cert(cert.unwrap()).await?;
        let _ = load_private_key(key.unwrap()).await?;
    }

    super::log_launch_and_open_browser(sc.as_ref());

    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }
    if sc.blueprint.server.enable_batch_requests {
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let sc = sc.clone();
            tokio::spawn(async move {
                let server = hyper::server::conn::http2::Builder::new(LocalExec)
                    .serve_connection(
                        io,
                        service_fn(move |req: Request<Incoming>| {
                            let state = sc.clone();
                            async move {
                                let (part, body) = req.into_parts();
                                let body = body.collect().await?.to_bytes();
                                let req = Request::from_parts(part, Full::new(body));
                                handle_request::<GraphQLBatchRequest>(req, state.app_ctx.clone())
                                    .await
                            }
                        }),
                    )
                    .await;
                if let Err(e) = server {
                    log::error!("An error occurred while handling a request: {e}");
                }
            });
        }
    } else {
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let sc = sc.clone();
            tokio::spawn(async move {
                let server = hyper::server::conn::http2::Builder::new(LocalExec)
                    .serve_connection(
                        io,
                        service_fn(move |req: Request<Incoming>| {
                            let state = sc.clone();
                            async move {
                                let (part, body) = req.into_parts();
                                let body = body.collect().await?.to_bytes();
                                let req = Request::from_parts(part, Full::new(body));
                                handle_request::<GraphQLRequest>(req, state.app_ctx.clone()).await
                            }
                        }),
                    )
                    .await;
                if let Err(e) = server {
                    log::error!("An error occurred while handling a request: {e}");
                }
            });
        }
    };
}
