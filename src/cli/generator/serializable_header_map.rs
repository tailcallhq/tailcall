use std::collections::HashMap;
use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
pub struct SerializableHeaderMap(pub HeaderMap);

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

        Ok(SerializableHeaderMap(header_map))
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
