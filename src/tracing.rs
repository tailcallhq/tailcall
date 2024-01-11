use std::{env, str::FromStr};

use tracing::Subscriber;
use tracing_subscriber::{filter::filter_fn, layer::SubscriberExt, Layer};

pub fn default_tracing() -> impl Subscriber {
  const LONG_ENV_FILTER_VAR_NAME: &str = "TAILCALL_LOG_LEVEL";
  const SHORT_ENV_FILTER_VAR_NAME: &str = "TC_LOG_LEVEL";

  let level = env::var(LONG_ENV_FILTER_VAR_NAME)
    .or(env::var(SHORT_ENV_FILTER_VAR_NAME))
    .ok()
    .map(|v| tracing::Level::from_str(&v).ok())
    .flatten()
    // use the log level from the env if there is one, otherwise use the default.
    .unwrap_or(tracing::Level::INFO);

  tracing_subscriber::fmt()
    .with_max_level(level)
    .compact()
    .finish()
    .with(default_filter_target())
}

pub fn default_filter_target<S: Subscriber>() -> impl Layer<S> {
  filter_fn(|metadata| metadata.target().starts_with("tailcall"))
}
