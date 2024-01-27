use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use protox::file::{FileResolver, GoogleFileResolver};
use url::Url;

use crate::{FileIO, HttpIO, ProtoPathResolver};

#[derive(Clone)]
pub struct NativeProtoPathResolver;

impl NativeProtoPathResolver {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl ProtoPathResolver for NativeProtoPathResolver {
    async fn resolve<'a>(
        &'a self,
        path: &'a str,
        http_io: Arc<dyn HttpIO>,
        file_io: Arc<dyn FileIO>,
    ) -> anyhow::Result<String> {
        if let Ok(file) = GoogleFileResolver::new().open_file(path) {
            return Ok(
                file.source()
                    .context("Unable to extract content of google well-known proto file")?
                    .to_string(),
            );
        }

        let proto_path = PathBuf::from(path);
        let proto_path = if proto_path.is_relative() {
            let dir = std::env::current_dir()?;
            dir.join(proto_path)
        } else {
            proto_path
        };

        let proto_path = proto_path
            .to_str()
            .map(String::from)
            .context("Unable to extract path")?;
        let source = match Url::parse(&proto_path) {
            Ok(url) => {
                let resp = http_io
                    .execute(reqwest::Request::new(reqwest::Method::GET, url))
                    .await?
                    .body
                    .to_vec();
                String::from_utf8(resp.to_vec())?
            }
            Err(_) => file_io.read(&proto_path).await?,
        };
        Ok(source)
    }
}
