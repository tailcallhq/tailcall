use std::collections::HashMap;
use std::hash::Hash;

#[allow(unused)]
#[derive(Default, Debug, Clone)]
pub struct Store<Key, Value> {
    map: HashMap<Key, Value>,
}

#[allow(unused)]
impl<K: PartialEq + Eq + Hash, V> Store<K, V> {
    pub fn new() -> Self {
        Store { map: HashMap::new() }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }
}
