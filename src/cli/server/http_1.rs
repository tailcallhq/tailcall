use std::sync::Arc;

use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use serde::de::DeserializeOwned;
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest, GraphQLRequestLike};
use crate::core::http::{handle_request, Request};

pub async fn start_http_1(
    sc: Arc<ServerConfig>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let mut builder = hyper::server::conn::http1::Builder::new();
    builder.keep_alive(true);
    super::log_launch(sc.as_ref());

    /* let mut _ty: impl GraphQLRequestLike + DeserializeOwned = GraphQLRequest;

     if sc.blueprint.server.enable_batch_requests {
         _ty = GraphQLBatchRequest;
     };*/

    /*    let make_svc_single_req = async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                handle_request::<GraphQLRequest>(req, sc.app_ctx.clone())
            }))
        };
        let make_svc_batch_req = async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                handle_request::<GraphQLBatchRequest>(req, sc.app_ctx.clone())
            }))
        };*/
    if let Some(sender) = server_up_sender {
        sender
            .send(())
            .or(Err(anyhow::anyhow!("Failed to send message")))?;
    }

    loop {
        let (stream, _) = listener.accept().await?;
        let app_ctx = sc.app_ctx.clone();


        let connection = builder
            .serve_connection(
                TokioIo::new(stream),
                service_fn(move |req| {
                    let app_ctx = app_ctx.clone();
                    async move {
                        let req = Request::from_hyper(req).await?;
                        handle_request::<
                            GraphQLRequest // TODO
                        >(req, app_ctx).await
                    }
                }),
            );
        tokio::spawn(async move {
            if let Err(err) = connection.await {
                println!("Error serving HTTP connection: {err:?}");
            }
        });
    }


    /*    let builder = hyper::Server::try_bind(&addr)
            .map_err(CLIError::from)?
            .http1_pipeline_flush(sc.app_ctx.blueprint.server.pipeline_flush);
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

        let result = server.map_err(CLIError::from);

        Ok(result?)*/
}
