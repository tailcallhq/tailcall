use std::collections::HashMap;

use reqwest::header::HeaderMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::mustache::Mustache;
use crate::core::path::PathString;

/// SerializableHeaderMap represents a wrapper around HeaderMap that supports serialization and deserialization,
/// and allows embedding Mustache templates in headers.
#[derive(Debug, PartialEq, Eq)]
pub struct SerializableHeaderMap(HeaderMap);

impl SerializableHeaderMap {
    pub fn new(headers: HeaderMap) -> Self {
        Self(headers)
    }

    /// Resolves Mustache templates within headers using the provided context values.
    pub fn resolve(mut self, context: &impl PathString) -> anyhow::Result<Self> {
        for header_value in self.0.values_mut() {
            *header_value = reqwest::header::HeaderValue::from_str(
                &Mustache::parse(header_value.to_str()?)?.render(context),
            )?;
        }
        Ok(self)
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SerializableHeaderMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize a HashMap<String, String>
        let map: HashMap<String, String> = Deserialize::deserialize(deserializer)?;
        // Convert the HashMap<String, String> to a HeaderMap
        let header_map: HeaderMap = (&map).try_into().map_err(serde::de::Error::custom)?;
        Ok(SerializableHeaderMap::new(header_map))
    }
}

impl Serialize for SerializableHeaderMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut headers_map: HashMap<&str, &str> = HashMap::new();
        for (key, value) in self.0.iter() {
            headers_map.insert(
                key.as_str(),
                value.to_str().map_err(serde::ser::Error::custom)?,
            );
        }
        headers_map.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, sync::Arc};

    use crate::core::{blueprint::Blueprint, config::ConfigReaderContext, EnvIO};
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

    use super::*;

    #[test]
    fn test_serialization_deserialization() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_str("Content-Type").unwrap(),
            HeaderValue::from_str("application/json").unwrap(),
        );
        headers.insert(
            HeaderName::from_str("Authorization").unwrap(),
            HeaderValue::from_str("Bearer eyJhbGciOiJIUzI1N").unwrap(),
        );

        let serializable_headers = SerializableHeaderMap::new(headers.clone());
        // Serialize it
        let serialized = serde_json::to_string(&serializable_headers).unwrap();
        // Deserialize it back
        let deserialized: SerializableHeaderMap = serde_json::from_str(&serialized).unwrap();
        // Check if the deserialized HeaderMap is equal to the original one
        assert_eq!(deserialized.0, headers);
    }

    struct EnvTest {
        env_vars: HashMap<String, String>,
    }

    impl EnvTest {
        fn init(env_vars: HashMap<String, String>) -> Self {
            Self { env_vars }
        }
    }

    impl EnvIO for EnvTest {
        fn get(&self, key: &str) -> Option<std::borrow::Cow<'_, str>> {
            self.env_vars.get(key).map(std::borrow::Cow::from)
        }
    }

    #[test]
    fn test_headers_with_mustache_template() -> anyhow::Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_str("Authorization").unwrap(),
            HeaderValue::from_str("Bearer {{env.TOKEN}}").unwrap(),
        );
        let headers = SerializableHeaderMap::new(headers);
        let mut runtime = crate::cli::runtime::init(&Blueprint::default());
        let mut env_vars = HashMap::new();
        env_vars.insert("TOKEN".to_owned(), "eyJhbGciOiJIUzI1N".to_owned());
        runtime.env = Arc::new(EnvTest::init(env_vars));

        let reader_context = ConfigReaderContext {
            runtime: &runtime,
            vars: &Default::default(),
            headers: HeaderMap::new(),
        };
        let resolved_headers = headers.resolve(&reader_context)?;
        let expected_header_value = "Bearer eyJhbGciOiJIUzI1N";
        assert_eq!(
            resolved_headers.headers().get("Authorization").unwrap(),
            expected_header_value
        );
        Ok(())
    }
}
