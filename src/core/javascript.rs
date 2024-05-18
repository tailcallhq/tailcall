use crate::core::{Response, WorkerIO};

pub struct DefaultJsRuntime;

#[async_trait::async_trait]
impl<A: Send + Sync + 'static, B> WorkerIO<A, B> for DefaultJsRuntime {
    async fn call(&self, _: String, _: A) -> anyhow::Result<Option<B>> {
        anyhow::bail!("JavaScript runtime is not supported in this build")
    }
}

#[derive(Debug)]
pub struct JsResponse(pub Response<String>);

#[derive(Debug)]
pub struct JsRequest(pub reqwest::Request);

#[derive(Debug)]
pub enum Event {
    Request(JsRequest),
}

#[derive(Debug)]
pub enum Command {
    Request(JsRequest),
    Response(JsResponse),
}
