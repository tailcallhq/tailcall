use tc_core::blueprint::Upstream;
use tc_core::valid::{Valid, ValidationError};

use super::TryFoldConfig;
use crate::config::Config;
use crate::try_fold::TryFold;

pub fn to_upstream<'a>() -> TryFold<'a, Config, Upstream, String> {
  TryFoldConfig::<Upstream>::new(|config, up| {
    let upstream = up.merge_right(config.upstream.clone());
    if let Some(ref base_url) = upstream.base_url {
      Valid::from(reqwest::Url::parse(base_url).map_err(|e| ValidationError::new(e.to_string())))
        .map_to(upstream.clone())
    } else {
      Valid::succeed(upstream.clone())
    }
  })
}
