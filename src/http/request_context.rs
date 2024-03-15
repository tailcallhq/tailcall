use std::collections::HashMap;
use std::num::NonZeroU64;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use async_graphql_value::ConstValue;
use cache_control::{Cachability, CacheControl};
use derive_setters::Setters;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tokio::sync::watch;

use crate::blueprint::{Server, Upstream};
use crate::data_loader::DataLoader;
use crate::graphql::GraphqlDataLoader;
use crate::grpc;
use crate::grpc::data_loader::GrpcDataLoader;
use crate::http::{AppContext, DataLoaderRequest, HttpDataLoader};
use crate::runtime::TargetRuntime;

#[derive(Setters)]
pub struct RequestContext {
    pub server: Server,
    pub upstream: Upstream,
    pub req_headers: HeaderMap,
    pub experimental_headers: HeaderMap,
    pub cookie_headers: Option<Arc<Mutex<HeaderMap>>>,
    pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
    pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
    pub grpc_data_loaders: Arc<Vec<DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>>>,
    pub min_max_age: Arc<Mutex<Option<i32>>>,
    pub cache_public: Arc<Mutex<Option<bool>>>,
    pub runtime: TargetRuntime,
    pub cache: Arc<RwLock<HashMap<u64, watch::Receiver<Option<ConstValue>>>>>,
}

impl RequestContext {
    fn set_min_max_age_conc(&self, min_max_age: i32) {
        *self.min_max_age.lock().unwrap() = Some(min_max_age);
    }
    pub fn get_min_max_age(&self) -> Option<i32> {
        *self.min_max_age.lock().unwrap()
    }

    pub fn set_cache_public_false(&self) {
        *self.cache_public.lock().unwrap() = Some(false);
    }

    pub fn is_cache_public(&self) -> Option<bool> {
        *self.cache_public.lock().unwrap()
    }

    pub fn set_min_max_age(&self, max_age: i32) {
        let min_max_age_lock = self.get_min_max_age();
        match min_max_age_lock {
            Some(min_max_age) if max_age < min_max_age => {
                self.set_min_max_age_conc(max_age);
            }
            None => {
                self.set_min_max_age_conc(max_age);
            }
            _ => {}
        }
    }

    pub fn set_cache_visibility(&self, cachability: &Option<Cachability>) {
        if let Some(Cachability::Private) = cachability {
            self.set_cache_public_false()
        }
    }

    pub fn set_cache_control(&self, cache_policy: CacheControl) {
        if let Some(max_age) = cache_policy.max_age {
            self.set_min_max_age(max_age.as_secs() as i32);
        }
        self.set_cache_visibility(&cache_policy.cachability);
        if Some(Cachability::NoCache) == cache_policy.cachability {
            self.set_min_max_age(-1);
        }
    }

    pub fn set_cookie_headers(&self, headers: &HeaderMap) {
        // TODO fix execution_spec test and use append method
        // to allow multiple set cookie
        if let Some(map) = &self.cookie_headers {
            let map = &mut map.lock().unwrap();

            // Check if the incoming headers contain 'set-cookie'
            if let Some(new_cookies) = headers.get("set-cookie") {
                let cookie_name = HeaderName::from_str("set-cookie").unwrap();

                // Check if 'set-cookie' already exists in our map
                if let Some(existing_cookies) = map.get(&cookie_name) {
                    // Convert the existing HeaderValue to a str, append the new cookies,
                    // and then convert back to a HeaderValue. If the conversion fails, we skip
                    // appending.
                    if let Ok(existing_str) = existing_cookies.to_str() {
                        if let Ok(new_cookies_str) = new_cookies.to_str() {
                            // Create a new value by appending the new cookies to the existing ones
                            let combined_cookies = format!("{}; {}", existing_str, new_cookies_str);

                            // Replace the old value with the new, combined value
                            map.insert(
                                cookie_name,
                                HeaderValue::from_str(&combined_cookies).unwrap(),
                            );
                        }
                    }
                } else {
                    // If 'set-cookie' does not already exist in our map, just insert the new value
                    map.insert(cookie_name, new_cookies.clone());
                }
            }
        }
    }

    pub async fn cache_get(&self, key: &u64) -> anyhow::Result<Option<ConstValue>> {
        self.runtime.cache.get(key).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn cache_insert(
        &self,
        key: u64,
        value: ConstValue,
        ttl: NonZeroU64,
    ) -> anyhow::Result<()> {
        self.runtime.cache.set(key, value, ttl).await
    }

    pub fn is_batching_enabled(&self) -> bool {
        self.upstream.is_batching_enabled()
    }
}

impl From<&AppContext> for RequestContext {
    fn from(app_ctx: &AppContext) -> Self {
        let cookie_headers = if app_ctx.blueprint.server.enable_set_cookie_header {
            Some(Arc::new(Mutex::new(HeaderMap::new())))
        } else {
            None
        };
        Self {
            server: app_ctx.blueprint.server.clone(),
            upstream: app_ctx.blueprint.upstream.clone(),
            req_headers: HeaderMap::new(),
            experimental_headers: HeaderMap::new(),
            cookie_headers,
            http_data_loaders: app_ctx.http_data_loaders.clone(),
            gql_data_loaders: app_ctx.gql_data_loaders.clone(),
            grpc_data_loaders: app_ctx.grpc_data_loaders.clone(),
            min_max_age: Arc::new(Mutex::new(None)),
            cache_public: Arc::new(Mutex::new(None)),
            runtime: app_ctx.runtime.clone(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, RwLock};

    use cache_control::Cachability;
    use hyper::HeaderMap;

    use crate::blueprint::{Server, Upstream};
    use crate::config::{self, Batch};
    use crate::http::RequestContext;

    impl Default for RequestContext {
        fn default() -> Self {
            let config_module = crate::config::ConfigModule::default();

            let crate::config::Config { upstream, .. } = config_module.config.clone();
            let server = Server::try_from(config_module).unwrap();
            let upstream = Upstream::try_from(upstream).unwrap();
            RequestContext {
                req_headers: HeaderMap::new(),
                experimental_headers: HeaderMap::new(),
                cookie_headers: None,
                server,
                runtime: crate::runtime::test::init(None),
                upstream,
                http_data_loaders: Arc::new(vec![]),
                gql_data_loaders: Arc::new(vec![]),
                grpc_data_loaders: Arc::new(vec![]),
                min_max_age: Arc::new(Mutex::new(None)),
                cache_public: Arc::new(Mutex::new(None)),
                cache: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[test]
    fn test_update_max_age_less_than_existing() {
        let req_ctx = RequestContext::default();
        req_ctx.set_min_max_age(120);
        req_ctx.set_min_max_age(60);
        assert_eq!(req_ctx.get_min_max_age(), Some(60));
    }

    #[test]
    fn test_update_max_age_greater_than_existing() {
        let req_ctx = RequestContext::default();
        req_ctx.set_min_max_age(60);
        req_ctx.set_min_max_age(120);
        assert_eq!(req_ctx.get_min_max_age(), Some(60));
    }

    #[test]
    fn test_update_max_age_no_existing_value() {
        let req_ctx = RequestContext::default();
        req_ctx.set_min_max_age(120);
        assert_eq!(req_ctx.get_min_max_age(), Some(120));
    }

    #[test]
    fn test_update_cache_visibility_private() {
        let req_ctx = RequestContext::default();
        req_ctx.set_cache_visibility(&Some(Cachability::Private));
        assert_eq!(req_ctx.is_cache_public(), Some(false));
    }

    #[test]
    fn test_update_cache_visibility_public() {
        let req_ctx: RequestContext = RequestContext::default();
        req_ctx.set_cache_visibility(&Some(Cachability::Public));
        assert_eq!(req_ctx.is_cache_public(), None);
    }

    #[test]
    fn test_is_batching_enabled_default() {
        // create ctx with default batch
        let config_module = config::ConfigModule::default();
        let server = Server::try_from(config_module.clone()).unwrap();
        let mut upstream = Upstream::try_from(config_module.upstream.clone()).unwrap();
        upstream.batch = Some(Batch::default());
        let req_ctx: RequestContext = RequestContext::default().upstream(upstream).server(server);

        assert!(req_ctx.is_batching_enabled());
    }
}
