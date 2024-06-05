use std::collections::HashMap;

#[allow(unused)]
#[derive(Default)]
pub struct Store<K, V> {
    map: HashMap<K, Data<K, V>>,
}

pub struct Data<K, V> {
    pub body: V,
    pub deferred: Vec<Defer<K>>,
}

pub struct Defer<K> {
    pub name: String,
    pub keys: Vec<K>,
}

#[allow(unused)]
impl<K, V> Store<K, V> {
    pub fn new() -> Self {
        Store { map: HashMap::new() }
    }

    pub fn get(&self, key: &K) -> Option<&Data<K, V>> {
        todo!()
    }

    pub fn insert(&mut self, key: K, value: Vec<Data<K, V>>) {
        todo!()
    }
}
