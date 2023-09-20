use ramhorns::Template;

use crate::endpoint_v2::Endpoint;
use crate::http::Method;

/// A template to quickly create a request
pub struct RequestTemplate<'a> {
  pub path: Template<'a>,
  pub query: Vec<(String, Template<'a>)>,
  pub method: Method,
  pub headers: Vec<(String, Template<'a>)>,
  pub body: Option<Template<'a>>,
}

impl RequestTemplate<'_> {}

impl<'a> TryFrom<&'a Endpoint> for RequestTemplate<'a> {
  type Error = anyhow::Error;
  fn try_from(endpoint: &'a Endpoint) -> anyhow::Result<Self> {
    let path = Template::new(endpoint.path.as_str())?;
    let query = endpoint
      .query
      .iter()
      .map(|(k, v)| Ok((k.to_owned(), Template::new(v.as_str())?)))
      .collect::<anyhow::Result<Vec<_>>>()?;
    let method = endpoint.method.clone();
    let headers = endpoint
      .headers
      .iter()
      .map(|(k, v)| Ok((k.as_str().into(), Template::new(v.to_str()?)?)))
      .collect::<anyhow::Result<Vec<_>>>()?;

    let body = if let Some(body) = &endpoint.body {
      Some(Template::new(body.as_str())?)
    } else {
      None
    };

    Ok(Self { path, query, method, headers, body })
  }
}
