use core::future::Future;
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;

use futures_util::FutureExt;
use reqwest_middleware::ClientWithMiddleware;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tower::limit::rate::Rate;
use tower::limit::RateLimit;
use tower::Service;

#[derive(Clone)]
pub enum HttpService {
  RateLimited(Arc<Mutex<RateLimit<HttpClientService>>>),
  Simple(Arc<Mutex<HttpClientService>>),
}

impl HttpService {
  pub fn simple(client: ClientWithMiddleware) -> Self {
    let service = HttpClientService::new(client);
    Self::Simple(Arc::new(Mutex::new(service)))
  }

  pub fn rate_limited(client: ClientWithMiddleware, num: u64, per: Duration) -> Self {
    let rate = Rate::new(num, per);
    let service = HttpClientService::new(client);
    let service = RateLimit::new(service, rate);
    Self::RateLimited(Arc::new(Mutex::new(service)))
  }

  pub async fn call(&self, req: reqwest::Request) -> anyhow::Result<reqwest::Response> {
    use HttpService::*;
    match self {
      RateLimited(service) => service.lock().await.call(req).await,
      Simple(service) => service.lock().await.call(req).await,
    }
  }
}

#[derive(Clone)]
pub struct HttpClientService {
  client: ClientWithMiddleware,
}

impl HttpClientService {
  pub fn new(client: ClientWithMiddleware) -> Self {
    HttpClientService { client }
  }
}

impl Service<reqwest::Request> for HttpClientService {
  type Response = reqwest::Response;
  type Error = anyhow::Error;
  type Future = HttpClientServiceFuture;

  fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
    todo!()
  }

  fn call(&mut self, req: reqwest::Request) -> Self::Future {
    let client = self.client.clone();
    let fut = tokio::spawn(async move { Ok(client.execute(req).await?) });
    HttpClientServiceFuture { fut }
  }
}

pub trait CustomFuture
where
  Self: Future<Output = anyhow::Result<reqwest::Response>> + Send + Sync,
{
}

impl<T> CustomFuture for T where T: Future<Output = anyhow::Result<reqwest::Response>> + Send + Sync {}

pub struct HttpClientServiceFuture {
  fut: JoinHandle<anyhow::Result<reqwest::Response>>,
}

impl Future for HttpClientServiceFuture {
  type Output = anyhow::Result<reqwest::Response>;

  fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
    match self.fut.poll_unpin(cx) {
      Poll::Pending => Poll::Pending,
      Poll::Ready(Ok(result)) => Poll::Ready(result),
      Poll::Ready(Err(err)) => Poll::Ready(Err(anyhow::anyhow!(err))),
    }
  }
}
