use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use tc_core::async_graphql_hyper::{GraphQLBatchRequest, GraphQLRequest};
use tc_core::http::handle_request;
use tokio::sync::oneshot;

use super::log_launch_and_open_browser;
use super::server_config::ServerConfig;
use crate::cli::CLIError;

pub async fn start_http_1(sc: Arc<ServerConfig>, server_up_sender: Option<oneshot::Sender<()>>) -> anyhow::Result<()> {
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
  let builder = hyper::Server::try_bind(&addr)
    .map_err(CLIError::from)?
    .http1_pipeline_flush(sc.server_context.blueprint.server.pipeline_flush);
  log_launch_and_open_browser(sc.as_ref());

  if let Some(sender) = server_up_sender {
    sender.send(()).or(Err(anyhow::anyhow!("Failed to send message")))?;
  }

  let server: Result<(), hyper::Error> = if sc.blueprint.server.enable_batch_requests {
    builder.serve(make_svc_batch_req).await
  } else {
    builder.serve(make_svc_single_req).await
  };

  let result = server.map_err(CLIError::from);

  Ok(result?)
}
