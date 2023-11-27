use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use super::{handle_request, log_launch};
use crate::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use crate::cli::CLIError;

pub async fn start_http_1(
  sc: Arc<ServerConfig>,
  tx: oneshot::Sender<bool>,
) -> std::prelude::v1::Result<(), anyhow::Error> {
  let addr = sc.addr();
  let make_svc_single_req = make_service_fn(|_conn| {
    let state = Arc::clone(&sc);
    async move {
      Ok::<_, anyhow::Error>(service_fn(move |req| {
        handle_request::<GraphQLRequest>(req, state.server_context.clone())
      }))
    }
  });

  let make_svc_batch_req = make_service_fn(|_conn| {
    let state = Arc::clone(&sc);
    async move {
      Ok::<_, anyhow::Error>(service_fn(move |req| {
        handle_request::<GraphQLBatchRequest>(req, state.server_context.clone())
      }))
    }
  });
  let builder = hyper::Server::try_bind(&addr).map_err(CLIError::from)?;
  tx.send(true).ok();
  log_launch(sc.as_ref());
  let server: std::prelude::v1::Result<(), hyper::Error> = if sc.blueprint.server.enable_batch_requests {
    builder.serve(make_svc_batch_req).await
  } else {
    builder.serve(make_svc_single_req).await
  };
  Ok(server.map_err(CLIError::from)?)
}
