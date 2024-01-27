use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Url;
use tailcall::{FileIO, HttpIO, ProtoPathResolver};

pub struct CloudflareProtoPathResolver {}

impl CloudflareProtoPathResolver {
    pub fn init() -> Self {
        Self {}
    }
}

#[async_trait]
impl ProtoPathResolver for CloudflareProtoPathResolver {
    async fn resolve<'a>(
        &'a self,
        path: &'a str,
        http_io: Arc<dyn HttpIO>,
        file_io: Arc<dyn FileIO>,
    ) -> anyhow::Result<String> {
        let source = match Url::parse(path) {
            Ok(url) => {
                let resp = http_io
                    .execute(reqwest::Request::new(reqwest::Method::GET, url))
                    .await?
                    .body
                    .to_vec();
                String::from_utf8(resp.to_vec())?
            }
            Err(_) => file_io.read(path).await?,
        };
        Ok(source)
    }
}
