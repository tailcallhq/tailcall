use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::blueprint::from_config::{to_json_schema_for_args, to_json_schema_for_field};
use crate::blueprint::transform::Transform;
use crate::blueprint::FieldDefinition;
use crate::config;
use crate::config::Config;
use crate::endpoint::Endpoint;
use crate::lambda::Lambda;
use crate::request_template::RequestTemplate;
use crate::try_fold::TryFolding;
use crate::valid::{Valid, ValidExtensions, ValidationError};

pub struct HttpFold {
  pub field: config::Field,
}

impl TryFolding for HttpFold {
  type Input = Config;
  type Value = FieldDefinition;
  type Error = String;

  fn try_fold(self, config: &Self::Input, mut field_definition: Self::Value) -> Valid<Self::Value, Self::Error> {
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

          field_definition.resolver = Some(Lambda::from_request_template(req_template).expression);

          Ok(field_definition)
        }
        None => Valid::fail("No base URL defined".to_string()),
      },
      None => Ok(field_definition),
    }
  }
}
