use core::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::future::FutureExt;
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

    pub async fn execute(&self, req: reqwest::Request) -> anyhow::Result<reqwest::Response> {
        use HttpService::*;
        match self {
            RateLimited(service) => {
                ServiceCaller {
                    service: &mut *service.lock().await,
                    state: Some(ServiceCallerState::PendingRequest(req)),
                }
                .await
            }
            Simple(service) => service.lock().await.call(req).await,
        }
    }
}

pub struct ServiceCaller<'a, S: Service<reqwest::Request>> {
    service: &'a mut S,
    state: Option<ServiceCallerState>,
}

enum ServiceCallerState {
    PendingRequest(reqwest::Request),
    RequestInProgress(HttpClientServiceFuture),
}

impl<'a, S: Service<reqwest::Request, Error = anyhow::Error, Future = HttpClientServiceFuture>>
    Future for ServiceCaller<'a, S>
{
    type Output = anyhow::Result<reqwest::Response>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use ServiceCallerState::*;
        let cur_state = self.state.take();

        match cur_state {
            Some(PendingRequest(req)) => match self.service.poll_ready(cx) {
                Poll::Pending => Poll::Ready(Err(anyhow::anyhow!("RATE_LIMIT_EXCEEDED"))),
                Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                Poll::Ready(Ok(())) => {
                    let fut = self.service.call(req);
                    self.state = Some(RequestInProgress(fut));
                    self.poll(cx)
                }
            },
            Some(RequestInProgress(mut fut)) => {
                let result = fut.poll_unpin(cx);
                match result {
                    Poll::Pending => {
                        self.state = Some(RequestInProgress(fut));
                        Poll::Pending
                    }
                    Poll::Ready(Ok(res)) => Poll::Ready(Ok(res)),
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                }
            }
            None => unreachable!(),
        }
        //
        // match (&mut self.state, self.service.poll_ready(cx)) {
        //   (PendingRequest(_), Poll::Pending) => Poll::Pending,
        //   (PendingRequest(_), Poll::Ready(Err(err))) => Poll::Ready(Err(err)),
        //   (PendingRequest(req), Poll::Ready(Ok(()))) => {
        //     let req = req.take().unwrap();
        //     self.state = RequestInProgress(self.service.call(req));
        //     match &mut self.state {
        //       RequestInProgress(fut) => fut.poll_unpin(cx),
        //       _ => unreachable!(),
        //     }
        //   }
        //   (RequestInProgress(fut), _) => fut.poll_unpin(cx),
        // }
        //
        // let fut = self.service.poll_ready(cx);
        // let state = self.state;
        // match state {
        //   (Poll::Pending, _) => Poll::Pending,
        //   (Poll::Ready(Ok(())), ) => {
        //     if self.fut.is_none() {
        //       let req = self.req.take().unwrap();
        //       self.fut = Some(self.service.call(req));
        //     }
        //
        //     let fut = self.fut.as_mut().unwrap();
        //     match pin!(fut).poll_unpin(cx) {
        //       Poll::Pending => Poll::Pending,
        //       Poll::Ready(Ok(res)) => Poll::Ready(Ok(res)),
        //       Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
        //     }
        //   }
        //   Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
        // }
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

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: reqwest::Request) -> Self::Future {
        let client = self.client.clone();
        let fut = tokio::spawn(async move { Ok(client.execute(req).await?) });
        HttpClientServiceFuture { fut }
    }
}

pub struct HttpClientServiceFuture {
    fut: JoinHandle<anyhow::Result<reqwest::Response>>,
}

impl Future for HttpClientServiceFuture {
    type Output = anyhow::Result<reqwest::Response>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.fut.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(err)) => Poll::Ready(Err(anyhow::anyhow!(err))),
        }
    }
}
