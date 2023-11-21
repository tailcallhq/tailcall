use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::config::KeyValues;
use crate::valid::{Valid, ValidationError};

pub fn to_headermap(headers: &KeyValues) -> Valid<HeaderMap, String> {
  Valid::from_iter(headers.iter(), |(k, v)| {
    let name =
      Valid::from(HeaderName::from_bytes(k.as_bytes()).map_err(|e| ValidationError::new(e.to_string()))).trace(k);

    let value =
      Valid::from(HeaderValue::from_str(v.as_str()).map_err(|e| ValidationError::new(e.to_string()))).trace(v);

    name.zip(value).map(|(name, value)| (name, value))
  })
  .map(HeaderMap::from_iter)
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use hyper::header::{HeaderName, HeaderValue};
  use hyper::HeaderMap;

  use super::to_headermap;
  use crate::config::KeyValues;

  #[test]
  fn valid_headers() -> Result<()> {
    let input: KeyValues = serde_json::from_str(r#"[{"key": "a", "value": "str"}, {"key": "b", "value": "123"}]"#)?;

    let headers = to_headermap(&input).to_result()?;

    assert_eq!(
      headers,
      HeaderMap::from_iter(vec![
        (HeaderName::from_bytes(b"a")?, HeaderValue::from_static("str")),
        (HeaderName::from_bytes(b"b")?, HeaderValue::from_static("123"))
      ])
    );

    Ok(())
  }

  #[test]
  fn not_valid_due_to_utf8() {
    let input: KeyValues =
      serde_json::from_str(r#"[{"key": "😅", "value": "str"}, {"key": "b", "value": "🦀"}]"#).unwrap();
    let error = to_headermap(&input).to_result().unwrap_err();

    // HeaderValue should be parsed just fine despite non-visible ascii symbols range
    // see https://github.com/hyperium/http/issues/519
    assert_eq!(
      error.to_string(),
      r"Validation Error
• invalid HTTP header name [😅]
"
    );
  }
}
