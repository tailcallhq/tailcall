use thiserror::Error;

use crate::{valid::Valid, http::RequestContext};

#[derive(Debug, Error, Clone, PartialEq)]
pub enum AuthError {
  #[error("Haven't found auth parameters")]
  Missing,
  #[error("Auth validation failed")]
  ValidationFailed
}

#[async_trait::async_trait]
pub(crate) trait AuthProvider {
  async fn validate(&self, req_ctx: &RequestContext) -> Valid<(), AuthError>;
}
