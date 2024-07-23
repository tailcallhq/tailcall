use std::num::NonZeroU64;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use async_graphql_value::ConstValue;
use cache_control::{Cachability, CacheControl};
use derive_setters::Setters;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::core::app_context::AppContext;
use crate::core::auth::context::AuthContext;
use crate::core::blueprint::{Server, Upstream};
use crate::core::data_loader::{DataLoader, DedupeResult};
use crate::core::graphql::GraphqlDataLoader;
use crate::core::grpc;
use crate::core::grpc::data_loader::GrpcDataLoader;
use crate::core::http::{DataLoaderRequest, HttpDataLoader};
use crate::core::ir::model::IoId;
use crate::core::ir::Error;
use crate::core::runtime::TargetRuntime;

#[derive(Setters)]
pub struct RequestContext {
    pub server: Server,
    pub upstream: Upstream,
    pub x_response_headers: Arc<Mutex<HeaderMap>>,
    pub cookie_headers: Option<Arc<Mutex<HeaderMap>>>,
    // A subset of all the headers received in the GraphQL Request that will be sent to the
    // upstream.
    pub allowed_headers: HeaderMap,
    pub auth_ctx: AuthContext,
    pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
    pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
    pub grpc_data_loaders: Arc<Vec<DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>>>,
    pub min_max_age: Arc<Mutex<Option<i32>>>,
    pub cache_public: Arc<Mutex<Option<bool>>>,
    pub runtime: TargetRuntime,
    pub cache: DedupeResult<IoId, ConstValue, Error>,
    pub dedupe_handler: Arc<DedupeResult<IoId, ConstValue, Error>>,
}

impl RequestContext {
    pub fn new(target_runtime: TargetRuntime) -> RequestContext {
        RequestContext {
            server: Default::default(),
            upstream: Default::default(),
            x_response_headers: Arc::new(Mutex::new(HeaderMap::new())),
            cookie_headers: None,
            http_data_loaders: Arc::new(vec![]),
            gql_data_loaders: Arc::new(vec![]),
            grpc_data_loaders: Arc::new(vec![]),
            min_max_age: Arc::new(Mutex::new(None)),
            cache_public: Arc::new(Mutex::new(None)),
            runtime: target_runtime,
            cache: DedupeResult::new(true),
            dedupe_handler: Arc::new(DedupeResult::new(false)),
            allowed_headers: HeaderMap::new(),
            auth_ctx: AuthContext::default(),
        }
    }
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

    pub async fn cache_get(&self, key: &IoId) -> anyhow::Result<Option<ConstValue>> {
        self.runtime.cache.get(key).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn cache_insert(
        &self,
        key: IoId,
        value: ConstValue,
        ttl: NonZeroU64,
    ) -> anyhow::Result<()> {
        self.runtime.cache.set(key, value, ttl).await
    }

    pub fn is_batching_enabled(&self) -> bool {
        self.upstream.is_batching_enabled()
    }

    /// Checks if experimental headers is enabled
    pub fn has_experimental_headers(&self) -> bool {
        !self.server.experimental_headers.is_empty()
    }

    /// Inserts the experimental headers into the x_response_headers map
    pub fn add_x_headers(&self, headers: &HeaderMap) {
        if self.has_experimental_headers() {
            let mut x_response_headers = self.x_response_headers.lock().unwrap();
            for name in &self.server.experimental_headers {
                if let Some(value) = headers.get(name) {
                    x_response_headers.insert(name, value.clone());
                }
            }
        }
    }

    /// Modifies existing headers to include the experimental headers
    pub fn extend_x_headers(&self, headers: &mut HeaderMap) {
        if self.has_experimental_headers() {
            let x_response_headers = &self.x_response_headers.lock().unwrap();
            for (header, value) in x_response_headers.iter() {
                headers.insert(header, value.clone());
            }
        }
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
            x_response_headers: Arc::new(Mutex::new(HeaderMap::new())),
            cookie_headers,
            allowed_headers: HeaderMap::new(),
            auth_ctx: (&app_ctx.auth_ctx).into(),
            http_data_loaders: app_ctx.http_data_loaders.clone(),
            gql_data_loaders: app_ctx.gql_data_loaders.clone(),
            grpc_data_loaders: app_ctx.grpc_data_loaders.clone(),
            min_max_age: Arc::new(Mutex::new(None)),
            cache_public: Arc::new(Mutex::new(None)),
            runtime: app_ctx.runtime.clone(),
            cache: DedupeResult::new(true),
            dedupe_handler: app_ctx.dedupe_handler.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use cache_control::Cachability;

    use crate::core::blueprint::{Server, Upstream};
    use crate::core::config::{self, Batch};
    use crate::core::http::RequestContext;

    impl Default for RequestContext {
        fn default() -> Self {
            let config_module = crate::core::config::ConfigModule::default();

            let upstream = Upstream::try_from(&config_module).unwrap();
            let server = Server::try_from(config_module).unwrap();
            RequestContext::new(crate::core::runtime::test::init(None))
                .upstream(upstream)
                .server(server)
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

    fn create_req_ctx_with_batch(batch: Batch) -> RequestContext {
        let config_module = config::ConfigModule::default();
        let mut upstream = Upstream::try_from(&config_module).unwrap();
        let server = Server::try_from(config_module).unwrap();
        upstream.batch = Some(batch);
        RequestContext::default().upstream(upstream).server(server)
    }

    #[test]
    fn test_is_batching_disabled_default() {
        let req_ctx = create_req_ctx_with_batch(Default::default());
        assert!(!req_ctx.is_batching_enabled());
    }

    #[test]
    fn test_is_batching_disabled_for_delay_zero() {
        let req_ctx = create_req_ctx_with_batch(Batch { delay: 0, ..Default::default() });
        assert!(!req_ctx.is_batching_enabled());
    }

    #[test]
    fn test_is_batching_disabled_for_max_size_none() {
        let req_ctx = create_req_ctx_with_batch(Batch { max_size: None, ..Default::default() });
        assert!(!req_ctx.is_batching_enabled());
    }

    #[test]
    fn test_is_batching_enabled() {
        let req_ctx =
            create_req_ctx_with_batch(Batch { delay: 1, max_size: Some(1), ..Default::default() });
        assert!(req_ctx.is_batching_enabled());
    }
}
