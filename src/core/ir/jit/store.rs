use std::collections::HashMap;

#[allow(unused)]
#[derive(Default)]
pub struct Store<K, V> {
    map: HashMap<K, Data<K, V>>,
}

struct Data<K, V> {
    body: V,
    deferred: Vec<Defer<K>>,
}

struct Defer<K> {
    name: String,
    keys: Vec<K>,
}

#[allow(unused)]
impl<K, V> Store<K, V> {
    pub fn new() -> Self {
        Store { map: HashMap::new() }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        todo!()
    }

    pub fn insert(&mut self, key: K, value: V) {
        todo!()
    }
}
