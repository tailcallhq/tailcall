use anyhow::{bail, Result};
use jsonwebtoken::jwk::JwkSet;
use url::Url;

use super::jwks::Jwks;
use super::jwks_remote::JwksRemote;
use super::jwt_verify::JwtClaim;
use crate::auth::error::Error;
use crate::blueprint;
use crate::init_context::InitContext;

pub enum JwksDecoder {
    Local(Jwks),
    Remote(JwksRemote),
}

impl JwksDecoder {
    pub fn try_new(options: &blueprint::JwtProvider, init_context: &InitContext) -> Result<Self> {
        match &options.jwks {
            blueprint::Jwks::Local(jwks) => {
                let jwks = jwks.render(init_context);
                if jwks.is_empty() {
                    bail!("JWKS data is empty");
                }

                let de = &mut serde_json::Deserializer::from_str(&jwks);
                let jwks: JwkSet = serde_path_to_error::deserialize(de)?;

                Ok(Self::Local(
                    Jwks::from(jwks).optional_kid(options.optional_kid),
                ))
            }
            blueprint::Jwks::Remote { url, max_age } => {
                let url = url.render(init_context);
                let url = Url::parse(&url)?;
                let decoder = JwksRemote::new(url, init_context.runtime.http.clone(), *max_age);

                Ok(Self::Remote(decoder.optional_kid(options.optional_kid)))
            }
        }
    }

    pub async fn decode(&self, token: &str) -> Result<JwtClaim, Error> {
        match self {
            JwksDecoder::Local(decoder) => decoder.decode(token),
            JwksDecoder::Remote(verifier) => verifier.decode(token).await,
        }
    }
}
