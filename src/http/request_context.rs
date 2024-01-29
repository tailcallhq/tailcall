use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};

use async_graphql_value::ConstValue;
use cache_control::{Cachability, CacheControl};
use derive_setters::Setters;
use hyper::HeaderMap;

use crate::blueprint::Server;
use crate::blueprint::Upstream;
// use crate::config::Upstream;
use crate::data_loader::DataLoader;
use crate::graphql::GraphqlDataLoader;
use crate::grpc::data_loader::GrpcDataLoader;
use crate::http::{AppContext, DataLoaderRequest, HttpDataLoader};
use crate::{grpc, EntityCache, EnvIO, HttpIO};

#[derive(Setters)]
pub struct RequestContext {
    // TODO: consider storing http clients where they are used i.e. expression and dataloaders
    pub h_client: Arc<dyn HttpIO>,
    // http2 only client is required for grpc in cases the server supports only http2
    // and the request will fail on protocol negotiation
    // having separate client for now looks like the only way to do with reqwest
    pub h2_client: Arc<dyn HttpIO>,
    pub server: Server,
    pub upstream: Upstream,
    pub req_headers: HeaderMap,
    pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
    pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
    pub grpc_data_loaders: Arc<Vec<DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>>>,
    pub min_max_age: Arc<Mutex<Option<i32>>>,
    pub cache_public: Arc<Mutex<Option<bool>>>,
    pub env_vars: Arc<dyn EnvIO>,
    pub cache: Arc<EntityCache>,
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

    pub async fn cache_get(&self, key: &u64) -> Option<ConstValue> {
        self.cache.get(key).await.ok()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn cache_insert(
        &self,
        key: u64,
        value: ConstValue,
        ttl: NonZeroU64,
    ) -> Option<ConstValue> {
        self.cache.set(key, value, ttl).await.ok()
    }

    pub fn is_batching_enabled(&self) -> bool {
        self.upstream.batch.is_some()
            && (self.upstream.get_delay() >= 1 || self.upstream.get_max_size() >= 1)
    }
}

impl From<&AppContext> for RequestContext {
    fn from(server_ctx: &AppContext) -> Self {
        Self {
            h_client: server_ctx.universal_http_client.clone(),
            h2_client: server_ctx.http2_only_client.clone(),
            server: server_ctx.blueprint.server.clone(),
            upstream: server_ctx.blueprint.upstream.clone(),
            req_headers: HeaderMap::new(),
            http_data_loaders: server_ctx.http_data_loaders.clone(),
            gql_data_loaders: server_ctx.gql_data_loaders.clone(),
            cache: server_ctx.cache.clone(),
            grpc_data_loaders: server_ctx.grpc_data_loaders.clone(),
            min_max_age: Arc::new(Mutex::new(None)),
            cache_public: Arc::new(Mutex::new(None)),
            env_vars: server_ctx.env_vars.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use cache_control::Cachability;
    use hyper::HeaderMap;

    use crate::blueprint::Server;
    use crate::blueprint::Upstream;
    use crate::cli::cache::NativeChronoCache;
    use crate::cli::{init_env, init_http, init_http2_only};
    use crate::config::{self};
    // use crate::config::Batch;
    use crate::blueprint::Batch;
    use crate::http::RequestContext;

    impl Default for RequestContext {
        fn default() -> Self {
            let crate::config::Config { server, upstream, .. } = crate::config::Config::default();
            //TODO: default is used only in tests. Drop default and move it to test.
            let server = Server::try_from(server).unwrap();
            let upstream = Upstream::default();
            let h_client = init_http(&upstream, None);
            let h2_client = init_http2_only(&upstream.clone(), None);
            RequestContext {
                req_headers: HeaderMap::new(),
                h_client,
                h2_client,
                server,
                upstream,
                http_data_loaders: Arc::new(vec![]),
                gql_data_loaders: Arc::new(vec![]),
                cache: Arc::new(NativeChronoCache::new()),
                grpc_data_loaders: Arc::new(vec![]),
                min_max_age: Arc::new(Mutex::new(None)),
                cache_public: Arc::new(Mutex::new(None)),
                env_vars: init_env(),
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
        let config = config::Config::default();
        let mut upstream = Upstream::default();
        upstream.batch = Some(Batch::default());
        let server = Server::try_from(config.server.clone()).unwrap();

        let req_ctx: RequestContext = RequestContext::default().upstream(upstream).server(server);

        assert!(req_ctx.is_batching_enabled());
    }
}
