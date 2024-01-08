use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use hyper::header::HeaderValue;
use hyper::Method;
use jwtk::jwk::{JwkSet, JwkSetVerifier};
use jwtk::HeaderAndClaims;
use reqwest::Request;
use url::Url;

use crate::auth::base::AuthError;
use crate::http::HttpClient;

struct JWKSCache {
  jwks: JwkSetVerifier,
  expiration: Instant,
}

pub struct RemoteJwksVerifier {
  url: Url,
  // as a trait object due to deep bubbling of generic definition
  // up to the entry point
  client: Arc<dyn HttpClient>,
  max_age: Duration,
  cache: RwLock<Option<JWKSCache>>,
  require_kid: bool,
}

impl RemoteJwksVerifier {
  pub fn new(url: Url, client: Arc<dyn HttpClient>, max_age: Duration) -> Self {
    Self { url, client, max_age, cache: RwLock::new(None), require_kid: true }
  }

  /// If called with `false`, subsequent `verify` calls will
  /// try all keys from the key set if a `kid` is not specified in the token.
  pub fn set_require_kid(&mut self, required: bool) {
    self.require_kid = required;
    if let Ok(Some(v)) = self.cache.get_mut() {
      v.jwks.set_require_kid(required);
    }
  }

  pub async fn verify(&self, token: &str) -> Result<HeaderAndClaims<()>, AuthError> {
    {
      let cache = self.cache.read().unwrap();

      if let Some(c) = cache.as_ref() {
        if c.expiration > Instant::now() {
          return c.jwks.verify(token).map_err(|_| AuthError::Invalid);
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
        return c.jwks.verify(token).map_err(|_| AuthError::Invalid);
      }
    }

    *cache = Some(JWKSCache {
      jwks: {
        let mut v = jwks.verifier();
        v.set_require_kid(self.require_kid);
        v
      },
      expiration: std::time::Instant::now() + self.max_age,
    });

    cache
      .as_ref()
      .unwrap()
      .jwks
      .verify(token)
      .map_err(|_| AuthError::Invalid)
  }

  async fn request_jwks(&self) -> anyhow::Result<JwkSet> {
    let mut request = Request::new(Method::GET, self.url.clone());

    request
      .headers_mut()
      .insert("accept", HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()));

    let response = self.client.execute_raw(request).await?;
    Ok(serde_json::from_value(response.json().await?)?)
  }
}
