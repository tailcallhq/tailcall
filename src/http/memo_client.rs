use super::HttpClient;
use anyhow::Result;
use http_cache_semantics::RequestLike;
use hyper::Uri;
use reqwest::Method;

#[allow(dead_code)]
pub struct MemoClient {
    client: HttpClient,
    cache: moka::sync::Cache<Uri, super::Response>,
}

impl MemoClient {
    #[allow(dead_code)]
    pub fn new(client: HttpClient) -> Self {
        Self { client, cache: moka::sync::Cache::new(u64::MAX) }
    }

    #[allow(dead_code)]
    pub async fn execute(&self, req: reqwest::Request) -> Result<super::Response> {
        if req.method() == Method::GET {
            let key = req.uri();
            let cached = self.cache.get(&key);
            if let Some(cached) = cached {
                Ok(cached)
            } else {
                let response = self.client.execute(req).await?;
                self.cache.insert(key, response.clone());
                Ok(response)
            }
        } else {
            Ok(self.client.execute(req).await?)
        }
    }
}
