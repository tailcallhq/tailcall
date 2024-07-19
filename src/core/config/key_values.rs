use std::collections::BTreeMap;
use std::ops::Deref;
use crate::core::is_default;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

type Value = (String, Option<bool>);

#[derive(Clone, Debug, Default, Eq, PartialEq, schemars::JsonSchema)]
pub struct KeyValues(pub BTreeMap<String, Value>);

impl Deref for KeyValues {
    type Target = BTreeMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<(String, Value)> for KeyValues {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        KeyValues(BTreeMap::from_iter(iter))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, schemars::JsonSchema)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "is_default")]
    #[serde(rename = "skipNull")]
    /// When specified in query params will skip the param whole value is null the default value of this argument is false
    pub skip_null: Option<bool>
}

// When we merge values, we do a merge right, which is to say that
// where both current and other have the same key, we use the value
// from other. This simplifies the merge_right_vars function in
// server.rs.
pub fn merge_key_value_vecs(current: &[KeyValue], other: &[KeyValue]) -> Vec<KeyValue> {
    let mut res = BTreeMap::new();

    for kv in current {
        res.insert(kv.key.to_owned(), (kv.value.to_owned(), kv.skip_null.to_owned()));
    }

    for kv in other {
        res.insert(kv.key.to_owned(), (kv.value.to_owned(), kv.skip_null.to_owned()));
    }

    res.into_iter()
        .map(|(k, (v,skip_null))| KeyValue { key: k, value: v, skip_null })
        .collect::<Vec<KeyValue>>()
}

impl Serialize for KeyValues {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let vec: Vec<KeyValue> = self
            .0
            .iter()
            .map(|(k, (v, skip_null))| KeyValue { key: k.clone(), value: v.clone(), skip_null: skip_null.to_owned() })
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
        let btree_map = vec.into_iter().map(|kv| (kv.key, (kv.value, kv.skip_null))).collect();
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
        kv.0.insert("a".to_string(), ("b".to_string(), None));
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
        let data = r#"[{"key":"a","value":"b", "skipNull": true}]"#;
        let kv: KeyValues = serde_json::from_str(data).unwrap();
        assert_eq!(kv.0["a"], ("b".to_string(), Some(true)));
    }

    #[test]
    fn test_default_keyvalues() {
        let kv = KeyValues::default();
        assert_eq!(kv.0.len(), 0);
    }

    #[test]
    fn test_deref() {
        let mut kv = KeyValues::default();
        kv.0.insert("a".to_string(), ("b".to_string(), None));
        // Using the deref trait
        assert_eq!(kv["a"], ("b".to_string(), None));
    }

    #[test]
    fn test_merge_with_both_empty() {
        let current = vec![];
        let other = vec![];
        let result = merge_key_value_vecs(&current, &other);
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_with_current_empty() {
        let current = vec![];
        let other = vec![KeyValue { key: "key1".to_string(), value: "value1".to_string(), skip_null: None }];
        let result = merge_key_value_vecs(&current, &other);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, "key1");
        assert_eq!(result[0].value, "value1");
    }

    #[test]
    fn test_merge_with_other_empty() {
        let current = vec![KeyValue { key: "key1".to_string(), value: "value1".to_string(), skip_null: None }];
        let other = vec![];
        let result = merge_key_value_vecs(&current, &other);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, "key1");
        assert_eq!(result[0].value, "value1");
    }

    #[test]
    fn test_merge_with_unique_keys() {
        let current = vec![KeyValue { key: "key1".to_string(), value: "value1".to_string(), skip_null: None }];
        let other = vec![KeyValue { key: "key2".to_string(), value: "value2".to_string(), skip_null: None }];
        let result = merge_key_value_vecs(&current, &other);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, "key1");
        assert_eq!(result[0].value, "value1");
        assert_eq!(result[1].key, "key2");
        assert_eq!(result[1].value, "value2");
    }

    #[test]
    fn test_merge_with_overlapping_keys() {
        let current = vec![KeyValue { key: "key1".to_string(), value: "value1".to_string(), skip_null: None }];
        let other = vec![KeyValue { key: "key1".to_string(), value: "value2".to_string(), skip_null: None }];
        let result = merge_key_value_vecs(&current, &other);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, "key1");
        assert_eq!(result[0].value, "value2");
    }
}
