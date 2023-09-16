use std::collections::{BTreeMap, HashMap};

use async_graphql::dataloader::{DataLoader, HashMapCache};
use derive_setters::Setters;
use hyper::HeaderMap;

use super::{memo_client::MemoClient, EndpointKey, HttpClient, HttpDataLoader, Response};

#[derive(Setters)]
pub struct RequestContext {
    // RC is required to support clone
    pub data_loader: DataLoader<HttpDataLoader, HashMapCache>,
    pub memo_client: MemoClient,
    pub client: HttpClient,
}

impl Default for RequestContext {
    fn default() -> Self {
        RequestContext::new(HttpClient::default(), &HeaderMap::new())
    }
}

fn to_btree(headers: &HeaderMap) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for (k, v) in headers.iter() {
        // Unwrap is safe here because we know the header is valid utf8
        map.insert(k.to_string(), v.to_str().unwrap().to_string());
    }
    map
}

impl RequestContext {
    pub fn new(client: HttpClient, headers: &HeaderMap) -> Self {
        Self {
            data_loader: HttpDataLoader::new(client.clone())
                .headers(to_btree(headers))
                .to_async_data_loader(),
            memo_client: MemoClient::new(client.clone()),
            client,
        }
    }

    #[allow(clippy::mutable_key_type)]
    pub fn get_cached_values(&self) -> HashMap<EndpointKey, Response> {
        #[allow(clippy::mutable_key_type)]
        self.data_loader.get_cached_values()
    }
}
