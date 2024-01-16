use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use hyper::header::HeaderValue;
use hyper::Method;
use jsonwebtoken::jwk::JwkSet;
use reqwest::Request;
use url::Url;

use super::jwks::Jwks;
use super::JwtClaims;
use crate::auth::base::AuthError;
use crate::HttpIO;

struct JWKSCache {
  jwks: Jwks,
  expiration: Instant,
}

pub struct RemoteJwksVerifier {
  url: Url,
  // as a trait object due to deep bubbling of generic definition
  // up to the entry point
  client: Arc<dyn HttpIO>,
  max_age: Duration,
  cache: RwLock<Option<JWKSCache>>,
  optional_kid: bool,
}

impl RemoteJwksVerifier {
  pub fn new(url: Url, client: Arc<dyn HttpIO>, max_age: Duration) -> Self {
    Self { url, client, max_age, cache: RwLock::new(None), optional_kid: false }
  }

  /// If called with `true`, subsequent `decode` calls will
  /// try all keys from the key set if a `kid` is not specified in the token.
  pub fn optional_kid(mut self, optional: bool) -> Self {
    self.optional_kid = optional;

    self
  }

  pub async fn decode(&self, token: &str) -> Result<JwtClaims, AuthError> {
    {
      let cache = self.cache.read().unwrap();

      if let Some(c) = cache.as_ref() {
        if c.expiration > Instant::now() {
          return c.jwks.decode(token);
        }
      }
    }

    let jwks = self
      .request_jwks()
      .await
      .map_err(|_| AuthError::ValidationCheckFailed)?;

    let mut cache = self.cache.write().unwrap();
    if let Some(c) = cache.as_ref() {
      if c.expiration > Instant::now() {
        return c.jwks.decode(token);
      }
    }

    *cache = Some(JWKSCache {
      jwks: {
        let v = Jwks::from(jwks);
        v.optional_kid(self.optional_kid)
      },
      expiration: std::time::Instant::now() + self.max_age,
    });

    cache
      .as_ref()
      .unwrap()
      .jwks
      .decode(token)
      .map_err(|_| AuthError::Invalid)
  }

  async fn request_jwks(&self) -> anyhow::Result<JwkSet> {
    let mut request = Request::new(Method::GET, self.url.clone());

    request
      .headers_mut()
      .insert("accept", HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()));

    let response = self.client.execute(request).await?;
    Ok(response.to_json()?.body)
  }
}
