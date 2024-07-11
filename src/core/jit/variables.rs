use std::collections::HashMap;

use serde::Deserialize;

/// Variables store
#[derive(Debug, Deserialize)]
pub struct Variables<Input>(HashMap<String, Input>);

impl<Input> Variables<Input> {
    pub fn get(&self, name: &str) -> Option<&Input> {
        self.0.get(name)
    }
}

impl<Input> Default for Variables<Input> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Input> FromIterator<(String, Input)> for Variables<Input> {
    fn from_iter<T: IntoIterator<Item = (String, Input)>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}
