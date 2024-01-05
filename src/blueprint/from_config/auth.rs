use std::sync::Arc;

use super::TryFoldConfig;
use crate::auth::jwt::JwtProvider;
use crate::config::{Auth, Config};
use crate::directive::DirectiveCodec;
use crate::http::DefaultHttpClient;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn to_auth<'a>() -> TryFold<'a, Config, Auth, String> {
  TryFoldConfig::<Auth>::new(|config, up| {
    let auth = up.merge_right(config.auth.clone());

    Valid::succeed(())
      .and_then(|_| {
        if let Some(jwt) = auth.clone().jwt {
          JwtProvider::new(jwt, Arc::new(DefaultHttpClient::new(&config.upstream)))
            .map_to(auth.clone())
            .unit()
            .trace("JWT")
        } else {
          Valid::succeed(())
        }
      })
      .map_to(auth.clone())
      .trace(Auth::trace_name().as_str())
  })
}
