use super::TryFoldConfig;
use crate::blueprint::RateLimit;
use crate::config::Config;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn to_rate_limit<'a>() -> TryFold<'a, Config, Option<RateLimit>, String> {
  TryFoldConfig::<Option<RateLimit>>::new(|config, up| {
    match up
      .map(Ok)
      .or(config.rate_limit.as_ref().map(RateLimit::try_from))
      .transpose()
    {
      Ok(rate_limit) => Valid::succeed(rate_limit),
      Err(err) => Valid::fail(err.to_string()),
    }
  })
}
