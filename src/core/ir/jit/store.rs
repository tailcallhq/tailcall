use std::collections::HashMap;
use std::hash::Hash;

use crate::core::ir::jit::model::FieldId;

#[allow(unused)]
#[derive(Default, Debug)]
pub struct Store<Key, Value> {
    map: HashMap<Key, Data<Key, Value>>,
}

#[derive(Debug)]
pub struct Data<K, V> {
    pub data: Option<V>,
    pub deferred: HashMap<FieldId, K>,
}

#[allow(unused)]
impl<K: PartialEq + Eq + Hash, V> Store<K, V> {
    pub fn new() -> Self {
        Store { map: HashMap::new() }
    }

    pub fn get(&self, key: &K) -> Option<&Data<K, V>> {
        self.map.get(key)
    }

    pub fn insert(&mut self, key: K, value: Data<K, V>) {
        match self.map.get_mut(&key) {
            Some(data) => {
                data.deferred.extend(value.deferred);
            }
            None => {
                self.map.insert(key, value);
            }
        }
    }
}
