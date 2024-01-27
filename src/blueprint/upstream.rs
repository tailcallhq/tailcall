use super::TryFoldConfig;
use crate::config::Batch;
use crate::config::Config;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};
use derive_setters::Setters;

#[derive(Clone, Debug, Setters)]
pub struct Upstream {
    pub pool_idle_timeout: u64,
    pub pool_max_idle_per_host: usize,
    pub keep_alive_interval: u64,
    pub keep_alive_timeout: u64,
    pub keep_alive_while_idle: bool,
    pub proxy: String,
    pub connect_timeout: u64,
    pub timeout: u64,
    pub tcp_keep_alive: u64,
    pub user_agent: String,
    pub allowed_headers: String,
    pub base_url: String,
    pub http_cache: bool,
    pub batch: Option<Batch>,
    pub http2_only: bool,
}

pub fn to_upstream<'a>() -> TryFold<'a, Config, Upstream, String> {
    TryFoldConfig::<Upstream>::new(|config, up| {
        let upstream = up.merge_right(config.upstream.clone());
        if let Some(ref base_url) = upstream.base_url {
            Valid::from(
                reqwest::Url::parse(base_url).map_err(|e| ValidationError::new(e.to_string())),
            )
            .map_to(upstream.clone())
        } else {
            Valid::succeed(upstream.clone())
        }
    })
}
