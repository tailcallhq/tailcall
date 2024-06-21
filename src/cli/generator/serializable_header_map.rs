use std::collections::HashMap;
use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
pub struct SerializableHeaderMap(HeaderMap);

impl SerializableHeaderMap {
    pub fn new(headers: HeaderMap) -> Self {
        Self(headers)
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
        let mut header_map = HeaderMap::new();
        for (key, value) in map {
            header_map.insert(
                HeaderName::from_str(&key).map_err(serde::de::Error::custom)?,
                HeaderValue::from_str(&value).map_err(serde::de::Error::custom)?,
            );
        }

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
            HeaderValue::from_str("Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyIjp7ImlkIjoxLCJ1c2VybmFtZSI6ImV4YW1wbGVfdXNlciJ9LCJpYXQiOjE3MTg5NTk1MTIsImV4cCI6MTcxODk2MzExMn0.UC9n2yn7XJYp9CGCRKgg3dm31Ax2lhwba5BupxVx-1").unwrap(),
        );

        let serializable_headers = SerializableHeaderMap::new(headers.clone());
        // Serialize it
        let serialized = serde_json::to_string(&serializable_headers).unwrap();
        // Deserialize it back
        let deserialized: SerializableHeaderMap = serde_json::from_str(&serialized).unwrap();
        // Check if the deserialized HeaderMap is equal to the original one
        assert_eq!(deserialized.0, headers);
    }
}
