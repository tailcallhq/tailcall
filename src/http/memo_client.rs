use std::{collections::HashMap, sync::Mutex};

use super::HttpClient;
use anyhow::Result;
use http_cache_semantics::RequestLike;
use hyper::Uri;
use reqwest::Method;

#[allow(dead_code)]
pub struct MemoClient {
    client: HttpClient,
    cache: Mutex<HashMap<Uri, super::Response>>,
}

impl MemoClient {
    #[allow(dead_code)]
    pub fn new(client: HttpClient) -> Self {
        Self { client, cache: Mutex::new(HashMap::new()) }
    }

    fn get(&self, key: &Uri) -> Option<super::Response> {
        self.cache.lock().unwrap().get(key).cloned()
    }

    fn insert(&self, key: Uri, value: super::Response) {
        self.cache.lock().unwrap().insert(key, value);
    }

    #[allow(dead_code)]
    pub async fn execute(&mut self, req: reqwest::Request) -> Result<super::Response> {
        if req.method() == Method::GET {
            let key = req.uri();
            let cached = self.get(&key);
            if let Some(cached) = cached {
                Ok(cached.clone())
            } else {
                let response = self.client.execute(req).await?;
                self.insert(key, response.clone());
                Ok(response)
            }
        } else {
            Ok(self.client.execute(req).await?)
        }
    }
}
