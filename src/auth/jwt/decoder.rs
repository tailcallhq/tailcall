use std::sync::Arc;

use super::jwks::Jwks;
use super::remote_jwks::RemoteJwksVerifier;
use super::JwtClaims;
use crate::auth::base::AuthError;
use crate::{blueprint, HttpIO};

pub enum JwksDecoder {
  Local(Jwks),
  Remote(RemoteJwksVerifier),
}

impl JwksDecoder {
  pub fn new(options: &blueprint::JwtProvider, client: Arc<dyn HttpIO>) -> Self {
    match &options.jwks {
      blueprint::Jwks::Local(jwks) => Self::Local(Jwks::from(jwks.clone()).optional_kid(options.optional_kid)),
      blueprint::Jwks::Remote { url, max_age } => {
        let decoder = RemoteJwksVerifier::new(url.clone(), client, *max_age);

        Self::Remote(decoder.optional_kid(options.optional_kid))
      }
    }
  }

  pub async fn decode(&self, token: &str) -> Result<JwtClaims, AuthError> {
    match self {
      JwksDecoder::Local(decoder) => decoder.decode(token),
      JwksDecoder::Remote(verifier) => verifier.decode(token).await,
    }
  }
}
