use super::{JsonLike, JsonObjectLike};

/// Used to perform Lens operations on JsonLike types
#[derive(Clone, Debug)]
pub enum Lens {
    Select(String),
    Compose(String, Box<Lens>),
}

impl Lens {
    /// Used to easy construct a select operation
    pub fn select(name: &str) -> Self {
        Self::Select(name.to_string())
    }

    /// Used to chain a path
    pub fn compose(self, name: &str) -> Self {
        Self::Compose(name.to_string(), Box::new(self))
    }

    /// Used to apply the lest to get a value
    pub fn get<'obj, Json>(&self, json: &'obj Json) -> Option<&'obj Json>
    where
        for<'json> Json: JsonLike<'json>,
    {
        if let Some(obj) = json.as_object() {
            match self {
                Lens::Select(key) => obj.get_key(key),
                Lens::Compose(key, rest) => {
                    let obj = obj.get_key(key);
                    obj.and_then(|obj| rest.get(obj))
                }
            }
        } else {
            Some(json)
        }
    }

    /// Used to apply the lens to remove a value
    pub fn remove<Json>(&self, json: &mut Json) -> Option<Json>
    where
        for<'json> Json: JsonLike<'json>,
    {
        if let Some(obj) = json.as_object_mut() {
            match self {
                Lens::Select(key) => obj.remove_key(key),
                Lens::Compose(key, rest) => {
                    let obj = obj.remove_key(key);
                    obj.and_then(|mut obj| rest.remove(&mut obj))
                }
            }
        } else {
            None
        }
    }

    /// Used to apply the lens to set a value
    pub fn set<Json>(&self, json: &mut Json, value: Json)
    where
        for<'json> Json: JsonLike<'json>,
    {
        if let Some(obj) = json.as_object_mut() {
            match self {
                Lens::Select(key) => {
                    obj.insert_key(key, value);
                }
                Lens::Compose(key, b) => {
                    if let Some(mut temp) = obj.remove_key(key) {
                        b.set(&mut temp, value);
                        obj.insert_key(key, temp);
                    }
                }
            }
        }
    }
}
