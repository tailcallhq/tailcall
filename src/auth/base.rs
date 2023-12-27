use hyper::{Body, Request};
use thiserror::Error;

use crate::valid::Valid;

#[derive(Debug, Error)]
pub enum AuthError {
  #[error("Haven't found auth parameters")]
  Missing,
}

#[async_trait::async_trait]
pub(crate) trait AuthProvider {
  async fn validate(&mut self, request: &Request<Body>) -> Valid<(), AuthError>;
}
