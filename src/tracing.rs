use std::env;
use std::str::FromStr;

use tracing::Subscriber;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;

pub fn default_tailcall_tracing() -> impl Subscriber {
    default_tracing().with(tailcall_filter_target())
}

pub fn default_crate_tracing(name: &'static str) -> impl Subscriber {
    default_tracing().with(filter_target(name))
}

pub fn default_tracing() -> impl Subscriber {
    const LONG_ENV_FILTER_VAR_NAME: &str = "TAILCALL_LOG_LEVEL";
    const SHORT_ENV_FILTER_VAR_NAME: &str = "TC_LOG_LEVEL";

    let level = env::var(LONG_ENV_FILTER_VAR_NAME)
        .or(env::var(SHORT_ENV_FILTER_VAR_NAME))
        .ok()
        .and_then(|v| tracing::Level::from_str(&v).ok())
        // use the log level from the env if there is one, otherwise use the default.
        .unwrap_or(tracing::Level::INFO);

    tracing_subscriber::fmt()
        .with_max_level(level)
        .without_time()
        .with_target(false)
        .compact()
        .finish()
}

pub fn tailcall_filter_target<S: Subscriber>() -> impl Layer<S> {
    filter_target("tailcall")
}

pub fn filter_target<S: Subscriber>(name: &'static str) -> impl Layer<S> {
    filter_fn(move |metadata| metadata.target().starts_with(name))
}
