use std::collections::HashMap;
use std::hash::Hash;

#[allow(unused)]
#[derive(Default, Debug)]
pub struct Store<K, V> {
    map: HashMap<K, Data<K, V>>,
}

#[derive(Debug)]
pub struct Data<K, V> {
    pub value: Option<V>,
    pub deferred: Vec<Defer<K>>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Defer<K> {
    pub name: String,
    pub keys: Vec<K>,
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
