use std::hash::{Hash, Hasher};

use async_graphql_value::ConstValue;

#[derive(Clone)]
pub struct HashableConstValue(pub ConstValue);

impl Hash for HashableConstValue {
  fn hash<H: Hasher>(&self, state: &mut H) {
    hash_const_value(&self.0, state)
  }
}

impl PartialEq for HashableConstValue {
  fn eq(&self, other: &Self) -> bool {
    eq_const_value(&self.0, &other.0)
  }
}

impl Eq for HashableConstValue {}

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

pub fn eq_const_value(lhs: &ConstValue, rhs: &ConstValue) -> bool {
  match (lhs, rhs) {
    (&ConstValue::Null, &ConstValue::Null) => true,
    (ConstValue::Boolean(lhs), ConstValue::Boolean(rhs)) => lhs.eq(rhs),
    (ConstValue::Enum(lhs), ConstValue::Enum(rhs)) => lhs.eq(rhs),
    (ConstValue::Number(lhs), ConstValue::Number(rhs)) => lhs.eq(rhs),
    (ConstValue::Binary(lhs), ConstValue::Binary(rhs)) => lhs.eq(rhs),
    (ConstValue::String(lhs), ConstValue::String(rhs)) => lhs.eq(rhs),
    (ConstValue::List(lhs), ConstValue::List(rhs)) => lhs.iter().zip(rhs.iter()).all(|(lhs, rhs)| lhs.eq(rhs)),
    (ConstValue::Object(lhs), ConstValue::Object(rhs)) => lhs
      .iter()
      .zip(rhs.iter())
      .all(|((lname, lvalue), (rname, rvalue))| lname.eq(rname) && lvalue.eq(rvalue)),
    _ => false,
  }
}
