use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use serde_yaml::Value;

use super::valid::{Valid, Validator};

pub trait MergeRight: Sized {
    fn merge_right(self, other: Self) -> Valid<Self, String>;
}

impl<A: MergeRight> MergeRight for Option<A> {
    fn merge_right(self, other: Self) -> Valid<Self, String> {
        let valid = match (self, other) {
            (Some(this), Some(that)) => return this.merge_right(that).map(Some),
            (None, Some(that)) => Some(that),
            (Some(this), None) => Some(this),
            (None, None) => None,
        };

        Valid::succeed(valid)
    }
}

impl<A: MergeRight + Default> MergeRight for Arc<A> {
    fn merge_right(self, other: Self) -> Valid<Self, String> {
        let l = Arc::into_inner(self);
        let r = Arc::into_inner(other);
        l.merge_right(r).map(|x| Arc::new(x.unwrap_or_default()))
    }
}

impl<A> MergeRight for Vec<A> {
    fn merge_right(mut self, other: Self) -> Valid<Self, String> {
        self.extend(other);
        Valid::succeed(self)
    }
}

impl<K, V> MergeRight for BTreeMap<K, V>
where
    K: Ord,
    V: Clone + MergeRight,
{
    fn merge_right(mut self, other: Self) -> Valid<Self, String> {
        Valid::from_iter(other, |(name, other_value)| match self.remove(&name) {
            Some(value) => value.merge_right(other_value).map(|value| (name, value)),
            None => Valid::succeed((name, other_value)),
        })
        .map(Self::from_iter)
    }
}

impl<V> MergeRight for BTreeSet<V>
where
    V: Ord,
{
    fn merge_right(mut self, other: Self) -> Valid<Self, String> {
        self.extend(other);
        Valid::succeed(self)
    }
}

impl<V> MergeRight for HashSet<V>
where
    V: Eq + std::hash::Hash,
{
    fn merge_right(mut self, other: Self) -> Valid<Self, String> {
        self.extend(other);
        Valid::succeed(self)
    }
}

impl<K, V> MergeRight for HashMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    fn merge_right(mut self, other: Self) -> Valid<Self, String> {
        self.extend(other);
        Valid::succeed(self)
    }
}

impl MergeRight for Value {
    fn merge_right(self, other: Self) -> Valid<Self, String> {
        let value = match (self, other) {
            (Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_), other) => other,
            (Value::Sequence(mut lhs), other) => match other {
                Value::Sequence(rhs) => {
                    lhs.extend(rhs);
                    Value::Sequence(lhs)
                }
                other => {
                    lhs.push(other);
                    Value::Sequence(lhs)
                }
            },
            (Value::Mapping(mut lhs), other) => match other {
                Value::Mapping(rhs) => {
                    return Valid::from_iter(rhs, |(key, value)| match lhs.remove(&key) {
                        Some(lhs_value) => lhs_value.merge_right(value).map(|value| (key, value)),
                        None => Valid::succeed((key, value)),
                    })
                    .map(|it| Value::Mapping(serde_yaml::Mapping::from_iter(it)))
                }
                Value::Sequence(mut rhs) => {
                    rhs.push(Value::Mapping(lhs));
                    Value::Sequence(rhs)
                }
                other => other,
            },
            (Value::Tagged(lhs), other) => match other {
                Value::Tagged(rhs) => {
                    if lhs.tag == rhs.tag {
                        let tag = lhs.tag;

                        return lhs.value.merge_right(rhs.value).map(|value| {
                            Value::Tagged(Box::new(serde_yaml::value::TaggedValue { tag, value }))
                        });
                    } else {
                        Value::Tagged(rhs)
                    }
                }
                other => other,
            },
        };

        Valid::succeed(value)
    }
}
