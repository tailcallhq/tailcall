use hyper::header::{HeaderName, HeaderValue};

use crate::valid::{Valid, ValidationError};

pub fn validate_headers(headers: Option<Vec<(String, String)>>) -> Valid<(), String> {
  if let Some(headers) = &headers {
    for (k, v) in headers {
      // Append header name and value to error message in case of error
      HeaderName::from_bytes(k.as_bytes()).map_err(|e| ValidationError::new(e.to_string()))?;
      HeaderValue::from_str(v.as_str()).map_err(|e| ValidationError::new(e.to_string()))?;
    }
  }

  Ok(())
}
