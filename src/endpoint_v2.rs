#![allow(clippy::too_many_arguments)]

use derive_setters::Setters;
use hyper::HeaderMap;
use reqwest::Request;

use crate::batch::Batch;
use crate::http::Method;
use crate::json::JsonSchema;
use crate::lambda::EvaluationContext;
use crate::mustache::Mustache;

#[derive(Clone, Debug, Setters)]
pub struct Endpoint {
  pub path: String,
  pub query: Vec<(String, String)>,

  pub method: Method,
  pub input: Option<JsonSchema>,
  pub output: Option<JsonSchema>,
  pub headers: HeaderMap,
  pub body: Option<Mustache>,
  pub description: Option<String>,
  pub batch: Option<Batch>,
  pub list: Option<bool>,
}

impl Endpoint {
  pub fn new(url: String) -> Endpoint {
    Self {
      path: url,
      query: Default::default(),
      method: Default::default(),
      input: Default::default(),
      output: Default::default(),
      headers: Default::default(),
      body: Default::default(),
      description: Default::default(),
      batch: Default::default(),
      list: Default::default(),
    }
  }

  pub fn to_request(&self, ctx: &EvaluationContext) -> anyhow::Result<Request> {
    let mut request = Request::new(reqwest::Method::from(&self.method), self.path.parse()?);
    request.headers_mut().extend(self.headers.clone());
    request.headers_mut().extend(ctx.req_ctx.req_headers.clone());

    Ok(request)
  }
}

#[cfg(test)]
mod tests {
  use crate::http::RequestContext;

  lazy_static::lazy_static! {
    static ref REQ_CTX: RequestContext = RequestContext::default();
  }

  mod to_request {

    use crate::endpoint_v2::tests::REQ_CTX;
    use crate::endpoint_v2::Endpoint;
    use crate::http::Method;
    use crate::lambda::EvaluationContext;

    #[test]
    fn test_method() {
      let context = EvaluationContext::new(&REQ_CTX);
      let endpoint = Endpoint::new("http://abc.com".into());
      let request = endpoint.to_request(&context).unwrap();
      assert_eq!(request.method(), reqwest::Method::GET);
    }

    #[test]
    fn test_method_put() {
      let context = EvaluationContext::new(&REQ_CTX);
      let endpoint = Endpoint::new("http://abc.com".into()).method(Method::PUT);
      let request = endpoint.to_request(&context).unwrap();
      assert_eq!(request.method(), reqwest::Method::PUT);
    }
  }

  mod url {
    use crate::endpoint_v2::tests::REQ_CTX;
    use crate::endpoint_v2::Endpoint;
    use crate::lambda::EvaluationContext;

    #[test]
    fn test_url() {
      let context = EvaluationContext::new(&REQ_CTX);
      let endpoint = Endpoint::new("http://abc.com".into());
      let request = endpoint.to_request(&context).unwrap();
      assert_eq!(request.url().as_str(), "http://abc.com/");
    }

    #[test]
    #[ignore]
    fn test_url_query_param() {
      let context = EvaluationContext::new(&REQ_CTX);
      let endpoint = Endpoint::new("http://abc.com?a=1&b=2".into());
      let request = endpoint.to_request(&context).unwrap();
      assert_eq!(request.url().as_str(), "http://abc.com/");
    }
  }

  mod headers {
    use hyper::HeaderMap;

    use crate::endpoint_v2::tests::REQ_CTX;
    use crate::endpoint_v2::Endpoint;
    use crate::http::RequestContext;
    use crate::lambda::EvaluationContext;
    #[test]
    fn test_headers() {
      let mut headers = HeaderMap::new();
      headers.insert("Foo", "Bar".parse().unwrap());

      let context = EvaluationContext::new(&REQ_CTX);

      let endpoint = Endpoint::new("http://abc.com".into()).headers(headers);
      let request = endpoint.to_request(&context).unwrap();
      assert_eq!(request.headers().get("Foo").unwrap(), "Bar");
    }

    #[test]
    fn test_ctx_headers() {
      let mut headers = HeaderMap::new();
      headers.insert("Foo", "Bar".parse().unwrap());

      let req_ctx = RequestContext::default().req_headers(headers);
      let context = EvaluationContext::new(&req_ctx);

      let endpoint = Endpoint::new("http://abc.com".into());
      let request = endpoint.to_request(&context).unwrap();
      assert_eq!(request.headers().get("Foo").unwrap(), "Bar");
    }
  }
}
