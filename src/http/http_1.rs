use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};

use super::server_config::ServerConfig;
use super::{handle_batch_request, handle_single_request, log_launch};
use crate::cli::CLIError;

pub async fn start_http_1(sc: Arc<ServerConfig>) -> std::prelude::v1::Result<(), anyhow::Error> {
  let addr = sc.addr();
  Ok(
    sc.tokio_runtime()?
      .spawn(async move {
        let sc_cloned = sc.clone();

        let make_svc_single_req = make_service_fn(|_conn| {
          let state = Arc::clone(&sc_cloned);
          async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
              handle_single_request(req, state.server_context.clone())
            }))
          }
        });

        let make_svc_batch_req = make_service_fn(|_conn| {
          let state = Arc::clone(&sc_cloned);
          async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
              handle_batch_request(req, state.server_context.clone())
            }))
          }
        });
        let builder = hyper::Server::try_bind(&addr).map_err(CLIError::from)?;

        let server: std::prelude::v1::Result<(), hyper::Error> = if sc_cloned.blueprint.server.enable_batch_requests {
          builder.serve(make_svc_batch_req).await
        } else {
          builder.serve(make_svc_single_req).await
        };

        log_launch(sc.as_ref());
        server.map_err(CLIError::from)
      })
      .await??,
  )
}
