use thiserror::Error;

use crate::http::RequestContext;
use crate::valid::Valid;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum AuthError {
  #[error("Haven't found auth parameters")]
  Missing,
  #[error("Couldn't validate auth request")]
  ValidationNotAccessible,
  #[error("Auth validation failed")]
  ValidationFailed,
}

#[async_trait::async_trait]
pub(crate) trait AuthProvider {
  async fn validate(&self, req_ctx: &RequestContext) -> Valid<(), AuthError>;
}
