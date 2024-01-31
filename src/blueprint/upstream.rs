use std::collections::BTreeSet;

use derive_setters::Setters;

use crate::config::{self, Batch};
use crate::valid::{Valid, ValidationError};

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
    pub http_cache: bool,
    pub batch: Option<Batch>,
    pub http2_only: bool,
}

impl Upstream {
    pub fn get_pool_idle_timeout(&self) -> u64 {
        self.pool_idle_timeout
    }
    pub fn get_pool_max_idle_per_host(&self) -> usize {
        self.pool_max_idle_per_host
    }
    pub fn get_keep_alive_interval(&self) -> u64 {
        self.keep_alive_interval
    }
    pub fn get_keep_alive_timeout(&self) -> u64 {
        self.keep_alive_timeout
    }
    pub fn get_keep_alive_while_idle(&self) -> bool {
        self.keep_alive_while_idle
    }
    pub fn get_connect_timeout(&self) -> u64 {
        self.connect_timeout
    }
    pub fn get_timeout(&self) -> u64 {
        self.timeout
    }
    pub fn get_tcp_keep_alive(&self) -> u64 {
        self.tcp_keep_alive
    }
    pub fn get_user_agent(&self) -> String {
        self.user_agent.clone()
    }
    pub fn get_enable_http_cache(&self) -> bool {
        self.http_cache
    }
    pub fn get_allowed_headers(&self) -> BTreeSet<String> {
        self.allowed_headers.clone()
    }
    pub fn get_delay(&self) -> usize {
        self.batch.clone().map_or(0, |b| b.delay)
    }

    pub fn get_max_size(&self) -> usize {
        self.batch.clone().map_or(100, |b| b.max_size)
    }

    pub fn get_http_2_only(&self) -> bool {
        self.http2_only
    }
}

impl Default for Upstream {
    fn default() -> Self {
        // NOTE: Using unwrap because try_from default will never fail
        Upstream::try_from(config::Upstream::default()).unwrap()
    }
}

impl TryFrom<crate::config::Upstream> for Upstream {
    type Error = ValidationError<String>;

    fn try_from(config_upstream: config::Upstream) -> Result<Self, Self::Error> {
        get_batch(&config_upstream)
            .zip(get_base_url(&config_upstream))
            .zip(get_proxy(&config_upstream))
            .map(|((batch, base_url), proxy)| Upstream {
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
                allowed_headers: (config_upstream).get_allowed_headers(),
                base_url,
                http_cache: (config_upstream).get_enable_http_cache(),
                batch,
                http2_only: (config_upstream).get_http_2_only(),
            })
            .to_result()
    }
}

fn get_batch(upstream: &config::Upstream) -> Valid<Option<Batch>, String> {
    upstream.batch.as_ref().map_or_else(
        || Valid::succeed(None),
        |batch| match batch {
            config::Batch { max_size: _, delay: _, headers: _ } => Valid::succeed(Some(Batch {
                max_size: (upstream).get_max_size(),
                delay: (upstream).get_delay(),
                headers: batch.headers.clone(),
            })),
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
