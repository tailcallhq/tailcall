use std::collections::HashMap;
use std::ops::Deref;

use hyper::header::HeaderName;
use serde::{Deserialize, Serialize};

use crate::core::config::KeyValue;
use crate::core::mustache::Mustache;
use crate::core::valid::{Valid, ValidationError, Validator};

#[derive(PartialEq, Clone, Default)]
pub struct MustacheHeaders(Vec<(HeaderName, Mustache)>);

impl std::fmt::Debug for MustacheHeaders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Vec<(HeaderName, Mustache)>> for MustacheHeaders {
    fn from(value: Vec<(HeaderName, Mustache)>) -> Self {
        Self(value)
    }
}

impl IntoIterator for MustacheHeaders {
    type Item = (HeaderName, Mustache);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for MustacheHeaders {
    type Target = Vec<(HeaderName, Mustache)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MustacheHeaders {
    pub fn new(headers: Vec<(HeaderName, Mustache)>) -> Self {
        MustacheHeaders(headers)
    }
}

impl Serialize for MustacheHeaders {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize_headers(self, serializer)
    }
}

impl<'de> Deserialize<'de> for MustacheHeaders {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: HashMap<String, String> = HashMap::deserialize(deserializer)?;
        let headers = map
            .into_iter()
            .filter_map(|(k, v)| {
                HeaderName::from_bytes(k.as_bytes())
                    .ok()
                    .map(|name| (name, Mustache::parse(&v)))
            })
            .collect();
        Ok(MustacheHeaders(headers))
    }
}

fn serialize_headers<S>(headers: &MustacheHeaders, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(headers.0.len()))?;
    for (k, v) in &headers.0 {
        map.serialize_entry(&k.to_string(), v)?;
    }
    map.end()
}

pub fn to_mustache_headers(headers: &[KeyValue]) -> Valid<MustacheHeaders, String> {
    Valid::from_iter(headers.iter(), |key_value| {
        let name = Valid::from(
            HeaderName::from_bytes(key_value.key.as_bytes())
                .map_err(|e| ValidationError::new(e.to_string())),
        )
        .trace(&key_value.key);

        let value =
            Valid::succeed(Mustache::parse(key_value.value.as_str())).trace(&key_value.value);

        name.zip(value).map(|(name, value)| (name, value))
    })
    .map(MustacheHeaders::new)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use hyper::header::HeaderName;

    use super::to_mustache_headers;
    use crate::core::config::KeyValue;
    use crate::core::mustache::Mustache;
    use crate::core::valid::Validator;

    #[test]
    fn valid_headers() -> Result<()> {
        let input: Vec<KeyValue> = serde_json::from_str(
            r#"[{"key": "a", "value": "str"}, {"key": "b", "value": "123"}]"#,
        )?;

        let headers = to_mustache_headers(&input).to_result()?;

        assert_eq!(
            headers,
            vec![
                (HeaderName::from_bytes(b"a")?, Mustache::parse("str")),
                (HeaderName::from_bytes(b"b")?, Mustache::parse("123"))
            ]
            .into()
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
