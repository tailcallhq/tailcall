use async_std::task::spawn_local;
use hyper::body::Bytes;
use reqwest::{Client, Request};
use tailcall::http::Response;
use tailcall::HttpIO;

pub struct WasmHttp {
    client: Client,
}

impl WasmHttp {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }
}
#[async_trait::async_trait]
impl HttpIO for WasmHttp {
    async fn execute(&self, request: Request) -> anyhow::Result<Response<Bytes>> {
        let client = self.client.clone();
        let method = request.method().clone();
        let url = request.url().clone();
        // TODO: remove spawn local
        let res = spawn_local(async move {
            let response = client.execute(request).await?.error_for_status()?;
            Response::from_reqwest(response).await
        })
        .await?;
        Ok(res)
    }
}
