use std::collections::BTreeMap;

use async_graphql::parser::types::Type;

/// A intermediary data structure that allows fast access to the type of a
/// variable by it's name.
#[derive(Debug, Clone)]
pub struct TypeMap(BTreeMap<String, Type>);

impl TypeMap {
    pub fn get(&self, key: &str) -> Option<&Type> {
        self.0.get(key)
    }

    pub fn new(map: BTreeMap<String, Type>) -> Self {
        Self(map)
    }
}

impl From<Vec<(&str, Type)>> for TypeMap {
    fn from(map: Vec<(&str, Type)>) -> Self {
        Self(map.iter().map(|a| (a.0.to_owned(), a.1.clone())).collect())
    }
}
