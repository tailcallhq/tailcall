use crate::core::{Response, WorkerIO};

pub struct DefaultJsRuntime;

#[async_trait::async_trait]
impl<A: Send + Sync + 'static, B> WorkerIO<A, B> for DefaultJsRuntime {
    async fn call(&self, _: &'async_trait str, _: &'async_trait A) -> anyhow::Result<Option<B>> {
        anyhow::bail!("JavaScript runtime is not supported in this build")
    }
}

#[derive(Debug)]
pub struct WorkerResponse(pub Response<String>);

#[derive(Debug)]
pub struct WorkerRequest(pub reqwest::Request);

impl Clone for WorkerRequest {
    fn clone(&self) -> Self {
        WorkerRequest(self.0.try_clone().unwrap())
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Request(WorkerRequest),
}

#[derive(Debug)]
pub enum Command {
    Request(WorkerRequest),
    Response(WorkerResponse),
}
