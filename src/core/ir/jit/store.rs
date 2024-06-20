use std::collections::HashMap;
use serde_json_borrow::Value;
use crate::core::ir::jit::model::FieldId;

pub struct Store {
    map: HashMap<FieldId, Data<'static>>,
}

pub enum Data<'a> {
    Value(Value<'a>),
    List(Vec<Value<'a>>)
}

impl Data<'_> {
    pub fn as_value(&self) -> Option<&Value> {
        match self {
            Data::Value(value) => Some(value),
            _ => None
        }
    }

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        match self {
            Data::List(list) => Some(list),
            _ => None
        }
    }
}

impl Store {
    pub fn new() -> Self {
        Store {
            map: HashMap::new()
        }
    }

    pub fn insert(&mut self, field_id: FieldId, data: Data<'static>) {
        self.map.insert(field_id, data);
    }

    pub fn get(&self, field_id: &FieldId) -> Option<&Data> {
        self.map.get(field_id)
    }
}