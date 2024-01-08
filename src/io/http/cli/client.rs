use std::time::Duration;
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use crate::config::Upstream;
use crate::http::HttpClientOptions;

pub fn make_client(upstream: &Upstream, options: HttpClientOptions) -> ClientWithMiddleware {
    let mut builder = Client::builder()
        .tcp_keepalive(Some(Duration::from_secs(upstream.get_tcp_keep_alive())))
        .timeout(Duration::from_secs(upstream.get_timeout()))
        .connect_timeout(Duration::from_secs(upstream.get_connect_timeout()))
        .http2_keep_alive_interval(Some(Duration::from_secs(upstream.get_keep_alive_interval())))
        .http2_keep_alive_timeout(Duration::from_secs(upstream.get_keep_alive_timeout()))
        .http2_keep_alive_while_idle(upstream.get_keep_alive_while_idle())
        .pool_idle_timeout(Some(Duration::from_secs(upstream.get_pool_idle_timeout())))
        .pool_max_idle_per_host(upstream.get_pool_max_idle_per_host())
        .user_agent(upstream.get_user_agent());
    if options.http2_only {
        builder = builder.http2_prior_knowledge();
    }
    if let Some(ref proxy) = upstream.proxy {
        builder = builder.proxy(reqwest::Proxy::http(proxy.url.clone()).expect("Failed to set proxy in http client"));
    }

    let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));
    if upstream.get_enable_http_cache() {
        client = client.with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: MokaManager::default(),
            options: HttpCacheOptions::default(),
        }))
    }
    client.build()
}