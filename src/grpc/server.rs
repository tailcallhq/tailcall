use hyper::service::{make_service_fn, service_fn};
use std::sync::Arc;

use crate::{blueprint::Blueprint, cli::CLIError, config::Config};

/// Initialize the gRPC server with TLS configuration if provided
pub async fn start_server(&config: Config) -> Result<()> {
  let blueprint = Blueprint::try_from(&config).map_err(CLIError::from)?;
  todo!()
}
