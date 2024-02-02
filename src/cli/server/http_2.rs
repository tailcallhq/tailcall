#![allow(clippy::too_many_arguments)]
use std::io::BufReader;
use std::sync::Arc;

use anyhow::Result;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::Request;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls_pki_types::{
    CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer, PrivatePkcs8KeyDer, PrivateSec1KeyDer,
};
use tokio::fs::File;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

use super::server_config::ServerConfig;
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::cli::CLIError;
use crate::config::TlsCert;
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

#[async_trait::async_trait]
pub trait TcpIO: Clone {
    type T: hyper::rt::Read + hyper::rt::Write + Unpin;
    async fn get_io(&self, option: Option<TlsAcceptor>, stream: TcpStream) -> Result<Self::T>;
}

#[derive(Clone, Default)]
pub struct NoTlsTcpIO;
#[derive(Clone, Default)]
pub struct TlsTcpIO;
#[async_trait::async_trait]
impl TcpIO for NoTlsTcpIO {
    type T = TokioIo<TcpStream>;

    async fn get_io(&self, _: Option<TlsAcceptor>, stream: TcpStream) -> Result<Self::T> {
        Ok(TokioIo::new(stream))
    }
}
#[async_trait::async_trait]
impl TcpIO for TlsTcpIO {
    type T = TokioIo<TlsStream<TcpStream>>;

    async fn get_io(
        &self,
        tls_acceptor: Option<TlsAcceptor>,
        stream: TcpStream,
    ) -> Result<Self::T> {
        Ok(TokioIo::new(
            tls_acceptor
                .ok_or(CLIError::new("Unable to create stream"))?
                .accept(stream)
                .await?,
        ))
    }
}

pub async fn start_http_2<T: TcpIO>(
    sc: Arc<ServerConfig>,
    cert: Option<TlsCert>,
    server_up_sender: Option<oneshot::Sender<()>>,
    tcp_io: T,
) -> Result<()>
where
    <T as TcpIO>::T: Send + 'static,
{
    let addr = sc.addr();
    let listener = TcpListener::bind(addr).await?;

    let mut tls_acceptor = None;

    if cert.is_some() {
        let tls_cert = cert.unwrap();
        let certs = load_cert(tls_cert.cert).await?;
        let key = load_private_key(tls_cert.key).await?;
        let mut server_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(CLIError::from)?;
        server_config.alpn_protocols = vec![
            b"h2".to_vec(),
            b"http/1.1".to_vec(),
            b"http/1.0".to_vec(),
            b"http/0.9".to_vec(),
        ];
        tls_acceptor = Some(tokio_rustls::TlsAcceptor::from(Arc::new(server_config)));
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
            let sc = sc.clone();
            let io = tcp_io.get_io(tls_acceptor.clone(), stream).await?;
            tokio::spawn(async move {
                let server = hyper::server::conn::http2::Builder::new(TokioExecutor::new())
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
            let sc = sc.clone();
            let io = tcp_io.get_io(tls_acceptor.clone(), stream).await?;
            tokio::spawn(async move {
                let server = hyper::server::conn::http2::Builder::new(TokioExecutor::new())
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
