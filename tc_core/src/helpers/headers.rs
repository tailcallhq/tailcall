use hyper::header::HeaderName;

use crate::blueprint::KeyValues;
use crate::mustache::Mustache;
use crate::valid::{Valid, ValidationError};

pub type MustacheHeaders = Vec<(HeaderName, Mustache)>;

pub fn to_headervec(headers: &KeyValues) -> Valid<Vec<(HeaderName, Mustache)>, String> {
  Valid::from_iter(headers.iter(), |(k, v)| {
    let name =
      Valid::from(HeaderName::from_bytes(k.as_bytes()).map_err(|e| ValidationError::new(e.to_string()))).trace(k);

    let value = Valid::from(Mustache::parse(v.as_str()).map_err(|e| ValidationError::new(e.to_string()))).trace(v);

    name.zip(value).map(|(name, value)| (name, value))
  })
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use hyper::header::HeaderName;
  use crate::blueprint::KeyValues;

  use super::to_headervec;
  use crate::mustache::Mustache;

  #[test]
  fn valid_headers() -> Result<()> {
    let input: KeyValues = serde_json::from_str(r#"[{"key": "a", "value": "str"}, {"key": "b", "value": "123"}]"#)?;

    let headers = to_headervec(&input).to_result()?;

    assert_eq!(
      headers,
      vec![
        (HeaderName::from_bytes(b"a")?, Mustache::parse("str")?),
        (HeaderName::from_bytes(b"b")?, Mustache::parse("123")?)
      ]
    );

    Ok(())
  }

  #[test]
  fn not_valid_due_to_utf8() {
    let input: KeyValues =
      serde_json::from_str(r#"[{"key": "😅", "value": "str"}, {"key": "b", "value": "🦀"}]"#).unwrap();
    let error = to_headervec(&input).to_result().unwrap_err();

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
