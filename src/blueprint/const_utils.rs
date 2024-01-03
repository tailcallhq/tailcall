use std::hash::{Hash, Hasher};

use async_graphql_value::ConstValue;

pub fn hash_const_value<H: Hasher>(const_value: &ConstValue, state: &mut H) {
  match const_value {
    ConstValue::Null => {}
    ConstValue::Boolean(val) => val.hash(state),
    ConstValue::Enum(name) => name.hash(state),
    ConstValue::Number(num) => num.hash(state),
    ConstValue::Binary(bytes) => bytes.hash(state),
    ConstValue::String(string) => string.hash(state),
    ConstValue::List(list) => list.iter().for_each(|val| hash_const_value(val, state)),
    ConstValue::Object(object) => {
      let mut tmp_list: Vec<_> = object.iter().collect();
      tmp_list.sort_by(|(key1, _), (key2, _)| key1.cmp(key2));
      tmp_list.iter().for_each(|(key, value)| {
        key.hash(state);
        hash_const_value(value, state);
      })
    }
  }
}
