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

            // Check if it's an error status
            if let Err(err) = response.error_for_status_ref() {
                let body_text = response.text().await?;
                // Create an error with the status code and add body content as context
                return Err(anyhow::Error::new(err.without_url()).context(body_text));
            }

            Response::from_reqwest(response).await
        })
        .await?;
        tracing::info!("{} {} {}", method, url, res.status.as_u16());
        Ok(res)
    }
}
