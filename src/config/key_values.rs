use std::collections::BTreeMap;

use serde::de::Deserializer;

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq)]
pub struct KeyValues(pub Vec<KeyValue>);

impl From<KeyValues> for BTreeMap<String, String> {
  fn from(value: KeyValues) -> Self {
    let mut map = BTreeMap::new();
    for KeyValue { key, value } in value.0 {
      map.insert(key, value);
    }
    map
  }
}

impl From<BTreeMap<String, String>> for KeyValues {
  fn from(value: BTreeMap<String, String>) -> Self {
    let mut kvs = Vec::new();
    for (key, value) in value {
      kvs.push(KeyValue { key, value });
    }
    KeyValues(kvs)
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq)]
pub struct KeyValue {
  pub key: String,
  pub value: String,
}
pub fn key_values_to_map<'de, D>(deserializer: D) -> Result<BTreeMap<String, String>, D::Error>
where
  D: Deserializer<'de>,
{
  let kvs = Option::<KeyValues>::deserialize(deserializer)?;
  Ok(kvs.unwrap_or_default().into())
}
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

pub fn map_to_key_values<S>(value: &BTreeMap<String, String>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let kvs = KeyValues(
    value
      .iter()
      .map(|(key, value)| KeyValue { key: key.to_string(), value: value.to_string() })
      .collect(),
  );
  kvs.serialize(serializer)
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

  use serde_json;

  use super::*;

  #[test]
  fn test_key_value_serde() {
    let kv = KeyValue { key: "name".to_string(), value: "Alice".to_string() };
    let serialized = serde_json::to_string(&kv).unwrap();
    assert_eq!(serialized, r#"{"key":"name","value":"Alice"}"#);

    let deserialized: KeyValue = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, kv);
  }

  #[test]
  fn test_key_values_serde() {
    let kvs = KeyValues(vec![
      KeyValue { key: "name".to_string(), value: "Alice".to_string() },
      KeyValue { key: "age".to_string(), value: "30".to_string() },
    ]);
    let serialized = serde_json::to_string(&kvs).unwrap();
    assert_eq!(
      serialized,
      r#"[{"key":"name","value":"Alice"},{"key":"age","value":"30"}]"#
    );

    let deserialized: KeyValues = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, kvs);
  }

  #[test]
  fn test_conversion_keyvalues_to_map() {
    let kvs = KeyValues(vec![
      KeyValue { key: "name".to_string(), value: "Alice".to_string() },
      KeyValue { key: "age".to_string(), value: "30".to_string() },
    ]);
    let map: BTreeMap<String, String> = kvs.into();
    assert_eq!(map.get("name"), Some(&"Alice".to_string()));
    assert_eq!(map.get("age"), Some(&"30".to_string()));
  }

  #[test]
  fn test_conversion_map_to_keyvalues() {
    let mut map = BTreeMap::new();
    map.insert("name".to_string(), "Alice".to_string());
    map.insert("age".to_string(), "30".to_string());

    let kvs: KeyValues = map.clone().into();
    assert!(kvs
      .0
      .contains(&KeyValue { key: "name".to_string(), value: "Alice".to_string() }));
    assert!(kvs
      .0
      .contains(&KeyValue { key: "age".to_string(), value: "30".to_string() }));

    let serialized_map = serde_json::to_string(&map).unwrap();
    let deserialized_map: BTreeMap<String, String> = serde_json::from_str(&serialized_map).unwrap();
    assert_eq!(map, deserialized_map);
  }

  #[test]
  fn test_key_values_to_map_function() {
    let serialized = r#"[{"key":"name","value":"Alice"},{"key":"age","value":"30"}]"#;
    let deserialized_map: Result<BTreeMap<String, String>, _> =
      key_values_to_map(&mut serde_json::Deserializer::from_str(serialized));
    assert!(deserialized_map.is_ok());
    let map = deserialized_map.unwrap();
    assert_eq!(map.get("name"), Some(&"Alice".to_string()));
    assert_eq!(map.get("age"), Some(&"30".to_string()));
  }
}
