use anyhow::Result;
use async_std::task::spawn_local;
use hyper::body::Bytes;
use reqwest::Client;
use tailcall::core::http::Response;
use tailcall::core::HttpIO;

#[derive(Clone)]
pub struct WasmHttp {
    client: Client,
}

impl Default for WasmHttp {
    fn default() -> Self {
        Self { client: Client::new() }
    }
}

impl WasmHttp {
    pub fn init() -> Self {
        let client = Client::new();
        Self { client }
    }
}

#[async_trait::async_trait]
impl HttpIO for WasmHttp {
    // HttpClientOptions are ignored in Cloudflare
    // This is because there is little control over the underlying HTTP client
    async fn execute(&self, request: reqwest::Request) -> Result<Response<Bytes>> {
        let client = self.client.clone();
        let method = request.method().clone();
        let url = request.url().clone();
        // TODO: remove spawn local
        let res = spawn_local(async move {
            let response = client.execute(request).await?;
            Response::from_reqwest_with_error_handling(response).await
        })
        .await?;
        tracing::info!("{} {} {}", method, url, res.status.as_u16());
        Ok(res)
    }
}
