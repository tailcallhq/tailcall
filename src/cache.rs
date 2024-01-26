use std::collections::HashMap;
use std::sync::Mutex;

pub struct Cache<K, V>(Mutex<HashMap<K, V>>);

impl<K, V> Cache<K, V>
where
    K: std::cmp::Eq,
    K: PartialEq,
    K: core::hash::Hash,
    V: std::clone::Clone,
{
    pub fn get(&self, key: &K) -> Option<V> {
        self.0.lock().unwrap().get(key).cloned()
    }

    pub fn insert(&self, key: K, value: V) {
        self.0.lock().unwrap().insert(key, value);
    }

    pub fn empty() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}
