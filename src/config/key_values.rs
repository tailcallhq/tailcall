use std::collections::BTreeMap;
use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Default, Eq, PartialEq, schemars::JsonSchema)]
pub struct KeyValues(pub BTreeMap<String, String>);

impl Deref for KeyValues {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

impl Serialize for KeyValues {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let vec: Vec<KeyValue> = self
            .0
            .iter()
            .map(|(k, v)| KeyValue { key: k.clone(), value: v.clone() })
            .collect();
        vec.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for KeyValues {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<KeyValue> = Vec::deserialize(deserializer)?;
        let btree_map = vec.into_iter().map(|kv| (kv.key, kv.value)).collect();
        Ok(KeyValues(btree_map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_empty_keyvalues() {
        let kv = KeyValues::default();
        let serialized = serde_json::to_string(&kv).unwrap();
        assert_eq!(serialized, "[]");
    }

    #[test]
    fn test_serialize_non_empty_keyvalues() {
        let mut kv = KeyValues::default();
        kv.0.insert("a".to_string(), "b".to_string());
        let serialized = serde_json::to_string(&kv).unwrap();
        assert_eq!(serialized, r#"[{"key":"a","value":"b"}]"#);
    }

    #[test]
    fn test_deserialize_empty_keyvalues() {
        let data = "[]";
        let kv: KeyValues = serde_json::from_str(data).unwrap();
        assert_eq!(kv, KeyValues::default());
    }

    #[test]
    fn test_deserialize_non_empty_keyvalues() {
        let data = r#"[{"key":"a","value":"b"}]"#;
        let kv: KeyValues = serde_json::from_str(data).unwrap();
        assert_eq!(kv.0["a"], "b");
    }

    #[test]
    fn test_default_keyvalues() {
        let kv = KeyValues::default();
        assert_eq!(kv.0.len(), 0);
    }

    #[test]
    fn test_deref() {
        let mut kv = KeyValues::default();
        kv.0.insert("a".to_string(), "b".to_string());
        // Using the deref trait
        assert_eq!(kv["a"], "b");
    }
}
