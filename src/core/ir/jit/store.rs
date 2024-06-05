use serde_json_borrow::OwnedValue;

use super::model::FieldId;

#[allow(unused)]
pub struct Store {
    map: Vec<(FieldId, OwnedValue)>,
}

#[allow(unused)]
impl Store {
    #[allow(unused)]
    pub fn empty() -> Self {
        Store { map: Vec::new() }
    }

    #[allow(unused)]
    pub fn join(caches: Vec<Store>) -> Self {
        let mut map = Vec::new();
        for cache in caches {
            map.extend(cache.map);
        }
        Store { map }
    }
    #[allow(unused)]
    pub fn get(&self, key: &FieldId) -> Option<&OwnedValue> {
        self.map.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn insert(&mut self, key: FieldId, value: OwnedValue) {
        self.map.push((key, value));
    }
}
