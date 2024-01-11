use super::TryFoldConfig;
use crate::config::opentelemetry::Opentelemetry;
use crate::config::Config;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn to_opentelemetry<'a>() -> TryFold<'a, Config, Opentelemetry, String> {
  TryFoldConfig::<Opentelemetry>::new(|config, up| Valid::succeed(up.merge_right(config.opentelemetry.clone())))
}
