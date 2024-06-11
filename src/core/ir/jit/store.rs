use std::collections::HashMap;
use std::hash::Hash;

use crate::core::ir::jit::model::FieldId;

#[allow(unused)]
#[derive(Default, Debug)]
pub struct Stores<Key, Value> {
    pub map: HashMap<FieldId, Store<Key, Value>>,
}

#[derive(Default, Debug, Clone)] // TODO: drop clone and store ref in synth
pub struct Store<Key, Value> {
    map: HashMap<Key, Data<Value>>,
}

#[derive(Debug, Clone)]
pub struct Data<V> {
    pub data: Option<V>,
}

#[allow(unused)]
impl<K: PartialEq + Eq + Hash, V> Stores<K, V> {
    pub fn new() -> Self {
        Stores { map: HashMap::new() }
    }
    pub fn get(&self, key: &FieldId) -> Option<&Store<K, V>> {
        self.map.get(key)
    }
    pub fn insert(&mut self, key: FieldId, store: Store<K, V>) {
        self.map.insert(key, store);
    }
}

impl<K: PartialEq + Eq + Hash, V> Store<K, V> {
    pub fn new() -> Self {
        Store { map: HashMap::new() }
    }

    pub fn get(&self, key: &K) -> Option<&Data<V>> {
        self.map.get(key)
    }

    pub fn insert(&mut self, key: K, value: Data<V>) {
        self.map.insert(key, value);
    }
}
