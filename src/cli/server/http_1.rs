use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use tokio::sync::oneshot;

use super::server_config::ServerConfig;
use crate::cli::CLIError;

pub async fn start_http_1(
    sc: Arc<ServerConfig>,
    server_up_sender: Option<oneshot::Sender<()>>,
) -> anyhow::Result<()> {
    let addr = sc.addr();
    let make_svc_req = make_service_fn(|_conn| {
        let state = sc.tailcall_executor.clone();
        async move { Ok::<_, anyhow::Error>(service_fn(move |req| state.clone().execute(req))) }
    });
    let builder = hyper::Server::try_bind(&addr)
        .map_err(CLIError::from)?
        .http1_pipeline_flush(sc.tailcall_executor.get_blueprint_server().pipeline_flush);
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
