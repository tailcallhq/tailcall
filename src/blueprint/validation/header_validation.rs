use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::config::KeyValues;
use crate::valid::{Valid, ValidExtensions, ValidationError};

pub fn validate_headers(headers: KeyValues) -> Valid<HeaderMap, String> {
  let mut header_map = HeaderMap::new();

  for header in headers.0.iter() {
    let k = HeaderName::from_bytes(header.0.as_bytes())
      .map_err(|e| ValidationError::new(e.to_string()))
      .trace("addResponseHeaders")
      .trace("key")
      .trace(header.0.as_str())?;
    let v = HeaderValue::from_str(header.1.as_str())
      .map_err(|e| ValidationError::new(e.to_string()))
      .trace("addResponseHeaders")
      .trace("value")
      .trace(header.1.as_str())?;

    header_map.insert(k, v);
  }

  Ok(header_map)
}
