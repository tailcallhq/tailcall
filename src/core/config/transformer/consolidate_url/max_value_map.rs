use std::collections::HashMap;
use std::hash::Hash;

/// A data structure that holds K and V, and allows query the max valued key.
pub struct MaxValueMap<K, V> {
    map: HashMap<K, V>,
    max_valued_key: Option<K>,
}

impl<K, V> Default for MaxValueMap<K, V>
where
    K: Eq + Hash + Clone,
    V: PartialOrd + Clone + std::ops::AddAssign,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> MaxValueMap<K, V>
where
    K: Eq + Hash + Clone,
    V: PartialOrd + Clone + std::ops::AddAssign,
{
    pub fn new() -> Self {
        MaxValueMap { map: HashMap::new(), max_valued_key: None }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some((_, max_value)) = self.get_max_pair() {
            if *max_value < value {
                self.max_valued_key = Some(key.to_owned());
            }
        } else {
            self.max_valued_key = Some(key.to_owned());
        }
        self.map.insert(key, value);
    }

    pub fn increment(&mut self, key: K, increment_by: V)
    where
        V: Clone + std::ops::Add<Output = V>,
    {
        if let Some(value) = self.map.get(&key) {
            self.insert(key, value.to_owned() + increment_by);
        } else {
            self.insert(key, increment_by);
        }
    }

    pub fn get_max_pair(&self) -> Option<(&K, &V)> {
        if let Some(ref key) = self.max_valued_key {
            return self.map.get_key_value(key);
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut max_value_map = MaxValueMap::new();
        max_value_map.insert("a", 10);
        max_value_map.insert("b", 20);

        assert_eq!(max_value_map.get_max_pair(), Some((&"b", &20)));
    }

    #[test]
    fn test_increment() {
        let mut max_value_map = MaxValueMap::new();
        max_value_map.insert("a", 10);
        max_value_map.increment("a", 15); // "a" becomes 25

        assert_eq!(max_value_map.get_max_pair(), Some((&"a", &25)));
    }

    #[test]
    fn test_get_max_pair_empty() {
        let max_value_map: MaxValueMap<String, i32> = MaxValueMap::new();
        assert_eq!(max_value_map.get_max_pair(), None);
    }
}