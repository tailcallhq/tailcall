use super::TryFoldConfig;
use crate::config::{self, Config};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};

use derive_setters::Setters;
use std::collections::BTreeSet;

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
    pub allowed_headers: BTreeSet<String>,
    pub base_url: String,
    pub http_cache: bool,
    pub batch: Option<Batch>,
    pub http2_only: bool,
}

#[derive(PartialEq, Eq, Clone, Debug, Setters, schemars::JsonSchema)]
pub struct Batch {
    pub max_size: usize,
    pub delay: usize,
    pub headers: BTreeSet<String>,
}

impl Default for Batch {
    fn default() -> Self {
        Batch { max_size: 100, delay: 0, headers: BTreeSet::new() }
    }
}

impl Default for Upstream {
    fn default() -> Self {
        // NOTE: Using unwrap because try_from default will never fail
        Upstream::try_from(config::Upstream::default()).unwrap()
    }
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
        self.user_agent
            .clone()
    }
    pub fn get_enable_http_cache(&self) -> bool {
        self.http_cache
    }
    pub fn get_allowed_headers(&self) -> BTreeSet<String> {
        self.allowed_headers.clone()
    }
    pub fn get_delay(&self) -> usize {
        self.batch.clone().unwrap().delay
    }

    pub fn get_max_size(&self) -> usize {
        self.batch.clone().unwrap().max_size
    }

    pub fn get_http_2_only(&self) -> bool {
        self.http2_only
    }
}    

impl TryFrom<crate::config::Upstream> for Upstream {
    type Error = ValidationError<String>;

    fn try_from(config_upstream: config::Upstream) -> Result<Self, Self::Error> {
        
        Ok(Upstream{
            pool_idle_timeout: (config_upstream).get_pool_idle_timeout(),
            pool_max_idle_per_host: (config_upstream).get_pool_max_idle_per_host(),
            keep_alive_interval: (config_upstream).get_keep_alive_interval(),
            keep_alive_timeout: (config_upstream).get_keep_alive_timeout(),
            keep_alive_while_idle: (config_upstream).get_keep_alive_while_idle(),
            connect_timeout: (config_upstream).get_connect_timeout(),
            timeout: (config_upstream).get_timeout(),
            tcp_keep_alive: (config_upstream).get_keep_alive_interval(),
            user_agent: (config_upstream).get_user_agent(),
            allowed_headers: (config_upstream).get_allowed_headers(),
            http2_only: (config_upstream).get_http_2_only(),
            http_cache: (config_upstream).get_enable_http_cache(),
            proxy: (config_upstream).get_enable_http_cache(),
            batch: (config_upstream).get_enable_http_cache(),
            base_url: (config_upstream).get_enable_http_cache(),
        })
    }
}

/*pub fn to_upstream<'a>() -> TryFold<'a, Config, Upstream, String> {
    TryFoldConfig::<Upstream>::new(|config, up| {
        let upstream = up.merge_right();
        if let ref base_url = upstream.base_url {
            Valid::from(
                reqwest::Url::parse(base_url).map_err(|e| ValidationError::new(e.to_string())),
            )
            .map_to(upstream.clone())
        } else {
            Valid::succeed(upstream.clone())
        }
    })
}*/