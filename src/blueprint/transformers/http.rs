use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::blueprint::from_config::{to_json_schema_for_args, to_json_schema_for_field};
use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::FieldDefinition;
use crate::config;
use crate::config::Config;
use crate::endpoint::Endpoint;
use crate::lambda::Lambda;
use crate::request_template::RequestTemplate;
use crate::valid::{ValidExtensions, ValidationError};

pub struct HttpTransform {
  pub field: config::Field,
}

impl From<HttpTransform> for Transform<Config, FieldDefinition, String> {
  fn from(value: HttpTransform) -> Self {
    Transform::new(move |config, field_definition| value.transform(config, field_definition).trace("@http"))
  }
}

impl HttpTransform {
  fn transform(self, config: &Config, mut b_field: FieldDefinition) -> Valid<FieldDefinition> {
    match self.field.http.as_ref() {
      Some(http) => match http
        .base_url
        .as_ref()
        .map_or_else(|| config.server.upstream.base_url.as_ref(), Some)
      {
        Some(base_url) => {
          let mut base_url = base_url.clone();
          if base_url.ends_with('/') {
            base_url.pop();
          }
          base_url.push_str(http.path.clone().as_str());
          let query = http.query.clone().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
          let output_schema = to_json_schema_for_field(&self.field, config);
          let input_schema = to_json_schema_for_args(&self.field.args, config);
          let mut header_map = HeaderMap::new();
          for (k, v) in http.headers.clone().iter() {
            header_map.insert(
              HeaderName::from_bytes(k.as_bytes()).map_err(|e| ValidationError::new(e.to_string()))?,
              HeaderValue::from_str(v.as_str()).map_err(|e| ValidationError::new(e.to_string()))?,
            );
          }
          let req_template = RequestTemplate::try_from(
            Endpoint::new(base_url.to_string())
              .method(http.method.clone())
              .query(query)
              .output(output_schema)
              .input(input_schema)
              .body(http.body.clone())
              .headers(header_map),
          )
          .map_err(|e| ValidationError::new(e.to_string()))?;

          b_field.resolver = Some(Lambda::from_request_template(req_template).expression);

          Valid::Ok(b_field)
        }
        None => Valid::fail("No base URL defined".to_string()),
      },
      None => Valid::Ok(b_field),
    }
  }
}
