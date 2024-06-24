use std::collections::HashMap;
use std::hash::Hash;

///
/// A special map that can hold two values of same type as key and any type of
/// value.
#[derive(Default)]
pub struct PairMap<A, V> {
    map: HashMap<(A, A), V>,
}

impl<A: PartialEq + Hash + Eq + Clone, V> PairMap<A, V> {
    pub fn add(&mut self, a1: A, a2: A, value: V) {
        self.map.insert((a1, a2), value);
    }

    pub fn get(&self, a1: &A, a2: &A) -> Option<&V> {
        if self.map.contains_key(&(a1.to_owned(), a2.to_owned())) {
            return self.map.get(&(a1.to_owned(), a2.to_owned()));
        } else if self.map.contains_key(&(a2.to_owned(), a1.to_owned())) {
            return self.map.get(&(a2.to_owned(), a1.to_owned()));
        }
        None
    }
}
