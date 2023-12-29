//! A JWKS server and token issuer. Based on https://github.com/blckngm/jwtk/blob/edc52da2eb4656437aa3f2f5097b6ebe696a85c7/examples/jwks.rs
//!
//! Reads private key from `example.key` (supports RSA, EC and Ed25519 keys).
//!
//! Jwks will be available at http://127.0.0.1:3000/jwks
//!
//! Tokens will be issued at http://127.0.0.1:3000/token

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Result};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use jwtk::jwk::{JwkSet, WithKid};
use jwtk::rsa::RsaAlgorithm;
use jwtk::{sign, HeaderAndClaims, PublicKeyToJwk, SomePrivateKey};

struct State {
  k: WithKid<SomePrivateKey>,
  jwks: JwkSet,
}

async fn jwks_handler(state: Arc<State>) -> Result<Response<Body>> {
  Ok(Response::new(Body::from(serde_json::to_string(&state.jwks)?)))
}

async fn token_handler(state: Arc<State>) -> Result<Response<Body>> {
  let mut token = HeaderAndClaims::new_dynamic();
  token
    .set_iss("me")
    .set_sub("you")
    .add_aud("them")
    .set_exp_from_now(Duration::from_secs(24 * 365 * 10 * 3600));
  let token = sign(&mut token, &state.k)?;

  Ok(Response::new(Body::from(
    serde_json::json!({
        "token": token,
    })
    .to_string(),
  )))
}

async fn handler(state: Arc<State>, request: Request<Body>) -> Result<Response<Body>> {
  match request.uri().path() {
    "/jwks" => jwks_handler(state).await,
    "/token" => token_handler(state).await,
    _ => bail!("No handler"),
  }
}

#[tokio::main]
async fn main() -> jwtk::Result<()> {
  let k = std::fs::read("examples/example.key")?;

  let k = SomePrivateKey::from_pem(
    &k,
    match std::env::var("RSA_ALGO").as_deref() {
      Ok(alg) => RsaAlgorithm::from_name(alg)?,
      _ => RsaAlgorithm::RS256,
    },
  )?;
  let k = WithKid::new_with_thumbprint_id(k)?;

  let k_public_jwk = k.public_key_to_jwk()?;
  let jwks = JwkSet { keys: vec![k_public_jwk] };

  let state = Arc::new(State { k, jwks });

  let make_service = make_service_fn(move |_conn| {
    let state = state.clone();

    let service = service_fn(move |req| handler(state.clone(), req));

    async move { Ok::<_, Infallible>(service) }
  });

  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

  let server = Server::bind(&addr).serve(make_service);

  if let Err(e) = server.await {
    eprintln!("server error: {}", e);
  }

  Ok(())
}
