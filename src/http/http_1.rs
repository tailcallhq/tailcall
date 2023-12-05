use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use tokio::sync::oneshot;

use super::server_config::{ServerConfig, ServerMessage};
use super::{handle_batch_request, handle_single_request, log_launch};
use crate::cli::CLIError;

pub async fn start_http_1(
  sc: Arc<ServerConfig>,
  server_up_sender: oneshot::Sender<ServerMessage>,
  shutdown_receiver: oneshot::Receiver<ServerMessage>,
) -> std::prelude::v1::Result<(), anyhow::Error> {
  let addr = sc.addr();
  let make_svc_single_req = make_service_fn(|_conn| {
    let state = Arc::clone(&sc);
    async move {
      Ok::<_, anyhow::Error>(service_fn(move |req| {
        handle_single_request(req, state.server_context.clone())
      }))
    }
  });

  let make_svc_batch_req = make_service_fn(|_conn| {
    let state = Arc::clone(&sc);
    async move {
      Ok::<_, anyhow::Error>(service_fn(move |req| {
        handle_batch_request(req, state.server_context.clone())
      }))
    }
  });
  let builder = hyper::Server::try_bind(&addr).map_err(CLIError::from)?;

  log_launch(sc.as_ref());
  server_up_sender.send(ServerMessage::ServerUp).ok();

  let server: std::prelude::v1::Result<(), hyper::Error> = if sc.blueprint.server.enable_batch_requests {
    builder
      .serve(make_svc_batch_req)
      .with_graceful_shutdown(async {
        log::info!("shutting down server");
        shutdown_receiver.await.ok().unwrap();
      })
      .await
  } else {
    builder
      .serve(make_svc_single_req)
      .with_graceful_shutdown(async {
        log::info!("shutting down server");
        shutdown_receiver.await.ok().unwrap();
      })
      .await
  };

  let result = server.map_err(CLIError::from);

  Ok(result?)
}
