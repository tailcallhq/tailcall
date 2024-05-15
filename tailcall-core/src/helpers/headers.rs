use hyper::header::HeaderName;

use crate::config::KeyValue;
use crate::mustache::Mustache;
use crate::valid::{Valid, ValidationError, Validator};

pub type MustacheHeaders = Vec<(HeaderName, Mustache)>;

pub fn to_mustache_headers(headers: &[KeyValue]) -> Valid<MustacheHeaders, String> {
    Valid::from_iter(headers.iter(), |key_value| {
        let name = Valid::from(
            HeaderName::from_bytes(key_value.key.as_bytes())
                .map_err(|e| ValidationError::new(e.to_string())),
        )
        .trace(&key_value.key);

        let value = Valid::from(
            Mustache::parse(key_value.value.as_str())
                .map_err(|e| ValidationError::new(e.to_string())),
        )
        .trace(&key_value.value);

        name.zip(value).map(|(name, value)| (name, value))
    })
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use hyper::header::HeaderName;

    use super::to_mustache_headers;
    use crate::config::KeyValue;
    use crate::mustache::Mustache;
    use crate::valid::Validator;

    #[test]
    fn valid_headers() -> Result<()> {
        let input: Vec<KeyValue> = serde_json::from_str(
            r#"[{"key": "a", "value": "str"}, {"key": "b", "value": "123"}]"#,
        )?;

        let headers = to_mustache_headers(&input).to_result()?;

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
        let input: Vec<KeyValue> =
            serde_json::from_str(r#"[{"key": "😅", "value": "str"}, {"key": "b", "value": "🦀"}]"#)
                .unwrap();
        let error = to_mustache_headers(&input).to_result().unwrap_err();

        // HeaderValue should be parsed just fine despite non-visible ascii symbols
        // range see https://github.com/hyperium/http/issues/519
        assert_eq!(
            error.to_string(),
            r"Validation Error
• invalid HTTP header name [😅]
"
        );
    }
}
