use std::collections::BTreeSet;

use derive_setters::Setters;

use crate::core::config::{self, Batch, ConfigModule};
use crate::core::valid::{Valid, ValidationError, Validator};

#[derive(PartialEq, Eq, Clone, Debug, schemars::JsonSchema)]
pub struct Proxy {
    pub url: String,
}

#[derive(PartialEq, Eq, Clone, Debug, Setters, schemars::JsonSchema)]
pub struct Upstream {
    pub pool_idle_timeout: u64,
    pub pool_max_idle_per_host: usize,
    pub keep_alive_interval: u64,
    pub keep_alive_timeout: u64,
    pub keep_alive_while_idle: bool,
    pub proxy: Option<Proxy>,
    pub connect_timeout: u64,
    pub timeout: u64,
    pub tcp_keep_alive: u64,
    pub user_agent: String,
    pub allowed_headers: BTreeSet<String>,
    pub base_url: Option<String>,
    pub http_cache: u64,
    pub batch: Option<Batch>,
    pub http2_only: bool,
    pub on_request: Option<String>,
}

impl Upstream {
    /// If the delay is set to 0, then batching is disabled. By default delay is
    /// set to 0.
    pub fn is_batching_enabled(&self) -> bool {
        if let Some(batch) = self.batch.as_ref() {
            batch.delay >= 1
        } else {
            false
        }
    }
}

impl Default for Upstream {
    fn default() -> Self {
        // NOTE: Using unwrap because try_from default will never fail
        Upstream::try_from(&ConfigModule::default()).unwrap()
    }
}

impl TryFrom<&ConfigModule> for Upstream {
    type Error = ValidationError<String>;

    fn try_from(config_module: &ConfigModule) -> Result<Self, Self::Error> {
        let config_upstream = config_module.upstream.clone();

        let mut allowed_headers = config_upstream.get_allowed_headers();

        if config_module.extensions().has_auth() {
            // force add auth specific headers to use it to make actual validation
            allowed_headers.insert(hyper::header::AUTHORIZATION.to_string());
        }

        get_batch(&config_upstream)
            .fuse(get_base_url(&config_upstream))
            .fuse(get_proxy(&config_upstream))
            .map(|(batch, base_url, proxy)| Upstream {
                pool_idle_timeout: (config_upstream).get_pool_idle_timeout(),
                pool_max_idle_per_host: (config_upstream).get_pool_max_idle_per_host(),
                keep_alive_interval: (config_upstream).get_keep_alive_interval(),
                keep_alive_timeout: (config_upstream).get_keep_alive_timeout(),
                keep_alive_while_idle: (config_upstream).get_keep_alive_while_idle(),
                proxy,
                connect_timeout: (config_upstream).get_connect_timeout(),
                timeout: (config_upstream).get_timeout(),
                tcp_keep_alive: (config_upstream).get_tcp_keep_alive(),
                user_agent: (config_upstream).get_user_agent(),
                allowed_headers,
                base_url,
                http_cache: (config_upstream).get_http_cache_size(),
                batch,
                http2_only: (config_upstream).get_http_2_only(),
                on_request: (config_upstream).get_on_request(),
            })
            .to_result()
    }
}

fn get_batch(upstream: &config::Upstream) -> Valid<Option<Batch>, String> {
    upstream.batch.as_ref().map_or_else(
        || Valid::succeed(None),
        |batch| {
            Valid::succeed(Some(Batch {
                max_size: Some((upstream).get_max_size()),
                delay: (upstream).get_delay(),
                headers: batch.headers.clone(),
            }))
        },
    )
}

fn get_base_url(upstream: &config::Upstream) -> Valid<Option<String>, String> {
    if let Some(ref base_url) = upstream.base_url {
        Valid::from(reqwest::Url::parse(base_url).map_err(|e| ValidationError::new(e.to_string())))
            .map_to(Some(base_url.clone()))
    } else {
        Valid::succeed(None)
    }
}

fn get_proxy(upstream: &config::Upstream) -> Valid<Option<Proxy>, String> {
    if let Some(ref proxy) = upstream.proxy {
        Valid::succeed(Some(Proxy { url: proxy.url.clone() }))
    } else {
        Valid::succeed(None)
    }
}
