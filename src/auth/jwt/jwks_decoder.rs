use anyhow::Result;

use super::jwks::Jwks;
use super::jwks_remote::JwksRemote;
use super::jwt_verify::JwtClaim;
use crate::auth::error::Error;
use crate::blueprint;
use crate::runtime::TargetRuntime;

pub enum JwksDecoder {
    Local(Jwks),
    Remote(JwksRemote),
}

impl JwksDecoder {
    pub fn new(options: blueprint::JwtProvider, runtime: &TargetRuntime) -> Self {
        match options.jwks {
            blueprint::Jwks::Local(jwks) => {
                Self::Local(Jwks::from(jwks).optional_kid(options.optional_kid))
            }
            blueprint::Jwks::Remote { url, max_age } => {
                let decoder = JwksRemote::new(url, runtime.http.clone(), max_age);

                Self::Remote(decoder.optional_kid(options.optional_kid))
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
