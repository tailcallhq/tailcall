use hyper::{Body, Method, Request, Response, StatusCode};
use tokio::net::TcpStream;

pub async fn handle(req: Request<hyper::Body>) -> anyhow::Result<Response<Body>> {
    tracing::info!("Proxy request: {:?}", req.uri());
    if Method::CONNECT == req.method() {
        if let Some(addr) = extract_addr(&req) {
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(mut upgraded) => {
                        if let Err(e) = async {
                            let mut server = TcpStream::connect(addr).await?;
                            let (from_client, from_server) =
                                tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;

                            tracing::info!(
                                "client wrote {} bytes and received {} bytes",
                                from_client,
                                from_server
                            );

                            Ok::<(), tokio::io::Error>(())
                        }
                        .await
                        {
                            tracing::error!("server io error: {}", e);
                        };
                    }
                    Err(e) => tracing::error!("upgrade error: {}", e),
                }
            });

            Ok(Response::new(Body::empty()))
        } else {
            tracing::error!("CONNECT host is not socket addr: {:?}", req.uri());
            let mut resp = Response::new(Body::from("CONNECT must be to a socket address"));
            *resp.status_mut() = StatusCode::BAD_REQUEST;

            Ok(resp)
        }
    } else {
        let host = req.uri().host().expect("uri has no host");
        let port = req.uri().port_u16().unwrap_or(80);

        let stream = TcpStream::connect((host, port)).await.unwrap();

        let (mut sender, conn) = hyper::client::conn::Builder::new()
            .http1_preserve_header_case(true)
            .http1_title_case_headers(true)
            .handshake(stream)
            .await?;

        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                tracing::error!("Connection failed: {:?}", err);
            }
        });

        let resp = sender.send_request(req).await?;
        Ok(resp)
    }
}

fn extract_addr(req: &Request<hyper::Body>) -> Option<String> {
    req.uri().authority().map(|auth| auth.to_string())
}
