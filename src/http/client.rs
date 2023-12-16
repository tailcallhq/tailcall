#[cfg(feature = "default")]
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};


use super::Response;
use crate::config::Upstream;


#[async_trait::async_trait]
pub trait HttpClient: Sync + Send {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response>;
}

#[async_trait::async_trait]
impl HttpClient for DefaultHttpClient {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response> {
    #[cfg(feature = "default")]
    return self.execute(req).await;

    /*  let client = reqwest::Client::new();
    let req = convert_reqwest_to_hyper(req).unwrap();
    let res = client.execute(req).await.unwrap();
    let body_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap().to_vec();
    println!("{}",String::from_utf8(body_bytes).unwrap());*/
    // let x = req.data().await.unwrap().unwrap();
    // println!("{:?}",x);
    // let x = self.client.execute(req).await?.error_for_stat us()?;
    // todo!();
    /*let x = async_std::task::block_on(async move {

      return match response {
        Ok(resource) => Ok(resource),
        Err(e) => {
          Err(anyhow::anyhow!("{}",e.to_string()))
        }
      }
    });*/
    Ok(Response::default())
  }
}

fn convert_reqwest_to_hyper(req: reqwest::Request) -> Result<hyper::Request<hyper::Body>, Box<dyn std::error::Error>> {
  let method = req.method().clone();
  let uri: hyper::Uri = req.url().as_str().parse()?;

  let mut hyper_req = hyper::Request::builder().method(method).uri(uri);

  for (key, value) in req.headers().iter() {
    hyper_req = hyper_req.header(key.as_str(), value.to_str().unwrap());
  }
  let body = if let Some(reqwest_body) = req.body() {
    let whole_body = reqwest_body.as_bytes().unwrap_or(&[]);
    hyper::Body::from(whole_body.to_vec())
  } else {
    hyper::Body::empty()
  };

  Ok(hyper_req.body(body)?)
}

#[derive(Clone)]
pub struct DefaultHttpClient {
  client: ClientWithMiddleware,
}

impl Default for DefaultHttpClient {
  fn default() -> Self {
    let upstream = Upstream::default();
    //TODO: default is used only in tests. Drop default and move it to test.
    DefaultHttpClient::new(&upstream)
  }
}

impl DefaultHttpClient {
  pub fn new(upstream: &Upstream) -> Self {
    let mut builder = reqwest::Client::builder();
    // .tcp_keepalive(Some(Duration::from_secs(upstream.get_tcp_keep_alive())))
    // .timeout(Duration::from_secs(upstream.get_timeout()))
    // .connect_timeout(Duration::from_secs(upstream.get_connect_timeout()))
    // .http2_keep_alive_interval(Some(Duration::from_secs(upstream.get_keep_alive_interval())))
    // .http2_keep_alive_timeout(Duration::from_secs(upstream.get_keep_alive_timeout()))
    // .http2_keep_alive_while_idle(upstream.get_keep_alive_while_idle())
    // .pool_idle_timeout(Some(Duration::from_secs(upstream.get_pool_idle_timeout())))
    // .pool_max_idle_per_host(upstream.get_pool_max_idle_per_host())
    // .user_agent(upstream.get_user_agent());
    #[cfg(feature = "default")]
    if let Some(ref proxy) = upstream.proxy {
      builder = builder.proxy(reqwest::Proxy::http(proxy.url.clone()).expect("Failed to set proxy in http client"));
    }

    let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));
    #[cfg(feature = "default")]
    if upstream.get_enable_http_cache() {
      client = client.with(Cache(HttpCache {
        mode: CacheMode::Default,
        manager: MokaManager::default(),
        options: HttpCacheOptions::default(),
      }))
    }

    DefaultHttpClient { client: client.build() }
  }

  pub async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response> {
    log::info!("{} {} ", request.method(), request.url());
    let response = self.client.execute(request).await?.error_for_status()?;
    let response = Response::from_response(response).await?;
    Ok(response)
  }
}
