use std::collections::BTreeMap;

use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::valid::{Valid, ValidationError};

pub fn validate_headers(headers: Option<Vec<BTreeMap<String, String>>>) -> Valid<HeaderMap, String> {
  let mut header_map = HeaderMap::new();

  if let Some(headers) = headers {
    for header in headers {
      // Do some validation here, extract name and value, don't just unwrap as it might panic

      if !header.contains_key("name") {
        return Err(ValidationError::new("Header name is missing".to_string()));
      }

      if !header.contains_key("value") {
        return Err(ValidationError::new("Header value is missing".to_string()));
      }

      let name = header.get("name").unwrap();
      let value = header.get("value").unwrap();

      let k = HeaderName::from_bytes(name.as_bytes()).map_err(|e| ValidationError::new(e.to_string()))?;
      let v = HeaderValue::from_str(value.as_str()).map_err(|e| ValidationError::new(e.to_string()))?;

      header_map.insert(k, v);
    }
  }
  Ok(header_map)
}
