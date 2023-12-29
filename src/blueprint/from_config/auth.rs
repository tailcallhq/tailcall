use super::TryFoldConfig;
use crate::auth::jwt::JwtProvider;
use crate::config::{Auth, Config};
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn to_auth<'a>() -> TryFold<'a, Config, Auth, String> {
  TryFoldConfig::<Auth>::new(|config, up| {
    let auth = up.merge_right(config.auth.clone());

    Valid::succeed(())
      .and_then(|_| {
        if let Some(jwt) = auth.clone().jwt {
          JwtProvider::parse(jwt).map_to(auth.clone()).unit().trace("JWT")
        } else {
          Valid::succeed(())
        }
      })
      .map_to(auth.clone())
      .trace("@auth")
  })
}
