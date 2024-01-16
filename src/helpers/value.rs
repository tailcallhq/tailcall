use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use async_graphql_value::ConstValue;

#[derive(Clone, Eq)]
pub struct HashableConstValue(pub ConstValue);

impl Hash for HashableConstValue {
  fn hash<H: Hasher>(&self, state: &mut H) {
    hash(&self.0, state)
  }
}

impl PartialEq for HashableConstValue {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

pub fn hash<H: Hasher>(const_value: &ConstValue, state: &mut H) {
  match const_value {
    ConstValue::Null => {}
    ConstValue::Boolean(val) => val.hash(state),
    ConstValue::Enum(name) => name.hash(state),
    ConstValue::Number(num) => num.hash(state),
    ConstValue::Binary(bytes) => bytes.hash(state),
    ConstValue::String(string) => string.hash(state),
    ConstValue::List(list) => list.iter().for_each(|val| hash(val, state)),
    ConstValue::Object(object) => {
      let mut tmp_list: Vec<_> = object.iter().collect();
      tmp_list.sort_by(|(key1, _), (key2, _)| key1.cmp(key2));
      tmp_list.iter().for_each(|(key, value)| {
        key.hash(state);
        hash(value, state);
      })
    }
  }
}

pub fn is_pair_comparable(lhs: &ConstValue, rhs: &ConstValue) -> bool {
  matches!(
    (lhs, rhs),
    (ConstValue::Null, ConstValue::Null)
      | (ConstValue::Boolean(_), ConstValue::Boolean(_))
      | (ConstValue::Enum(_), ConstValue::Enum(_))
      | (ConstValue::Number(_), ConstValue::Number(_))
      | (ConstValue::Binary(_), ConstValue::Binary(_))
      | (ConstValue::String(_), ConstValue::String(_))
      | (ConstValue::List(_), ConstValue::List(_))
  )
}

pub fn is_list_comparable(list: &[ConstValue]) -> bool {
  list
    .iter()
    .zip(list.iter().skip(1))
    .all(|(lhs, rhs)| is_pair_comparable(lhs, rhs))
}

pub fn compare(lhs: &ConstValue, rhs: &ConstValue) -> Option<Ordering> {
  Some(match (lhs, rhs) {
    (ConstValue::Null, ConstValue::Null) => Ordering::Equal,
    (ConstValue::Boolean(lhs), ConstValue::Boolean(rhs)) => lhs.partial_cmp(rhs)?,
    (ConstValue::Enum(lhs), ConstValue::Enum(rhs)) => lhs.partial_cmp(rhs)?,
    (ConstValue::Number(lhs), ConstValue::Number(rhs)) => lhs
      .as_f64()
      .partial_cmp(&rhs.as_f64())
      .or(lhs.as_i64().partial_cmp(&rhs.as_i64()))
      .or(lhs.as_u64().partial_cmp(&rhs.as_u64()))?,
    (ConstValue::Binary(lhs), ConstValue::Binary(rhs)) => lhs.partial_cmp(rhs)?,
    (ConstValue::String(lhs), ConstValue::String(rhs)) => lhs.partial_cmp(rhs)?,
    (ConstValue::List(lhs), ConstValue::List(rhs)) => lhs
      .iter()
      .zip(rhs.iter())
      .find_map(|(lhs, rhs)| compare(lhs, rhs).filter(|ord| ord != &Ordering::Equal))
      .unwrap_or(lhs.len().partial_cmp(&rhs.len())?),
    _ => None?,
  })
}

pub fn try_f64_operation<F>(lhs: &ConstValue, rhs: &ConstValue, f: F) -> Option<ConstValue>
where
  F: Fn(f64, f64) -> f64,
{
  match (lhs, rhs) {
    (ConstValue::Number(lhs), ConstValue::Number(rhs)) => {
      lhs.as_f64().and_then(|lhs| rhs.as_f64().map(|rhs| f(lhs, rhs).into()))
    }
    _ => None,
  }
}

pub fn try_i64_operation<F>(lhs: &ConstValue, rhs: &ConstValue, f: F) -> Option<ConstValue>
where
  F: Fn(i64, i64) -> i64,
{
  match (lhs, rhs) {
    (ConstValue::Number(lhs), ConstValue::Number(rhs)) => {
      lhs.as_i64().and_then(|lhs| rhs.as_i64().map(|rhs| f(lhs, rhs).into()))
    }
    _ => None,
  }
}

pub fn try_u64_operation<F>(lhs: &ConstValue, rhs: &ConstValue, f: F) -> Option<ConstValue>
where
  F: Fn(u64, u64) -> u64,
{
  match (lhs, rhs) {
    (ConstValue::Number(lhs), ConstValue::Number(rhs)) => {
      lhs.as_u64().and_then(|lhs| rhs.as_u64().map(|rhs| f(lhs, rhs).into()))
    }
    _ => None,
  }
}
