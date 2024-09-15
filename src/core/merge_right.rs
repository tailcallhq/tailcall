use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use serde_yaml::Value;
use crate::core::valid::{Valid, Validator};

pub trait MergeRight {
    fn merge_right(self, other: Self) -> Valid<Self, String>;
}

impl<A: MergeRight> MergeRight for Option<A> {
    fn merge_right(self, other: Self) -> Valid<Self, String> {
        let valid = match (self, other) {
            (Some(this), Some(that)) => Some(this.merge_right(that)),
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
        let valid = Arc::new(l.merge_right(r).unwrap_or_default());
        Valid::succeed(valid)
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
        for (other_name, mut other_value) in other {
            if let Some(self_value) = self.remove(&other_name) {
                other_value = self_value.merge_right(other_value);
            }

            self.insert(other_name, other_value);
        }
        Valid::succeed(self)
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
    fn merge_right(mut self, other: Self) -> Self {
        self.extend(other);
        self
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
        match (self, other) {
            (Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_), other) => Valid::succeed(other),
            (Value::Sequence(mut lhs), other) => match other {
                Value::Sequence(rhs) => {
                    lhs.extend(rhs);
                   Valid::succeed( Value::Sequence(lhs))
                }
                other => {
                    lhs.push(other);
                    Valid::succeed(Value::Sequence(lhs))
                }
            },
            (Value::Mapping(mut lhs), other) => match other {
                Value::Mapping(rhs) => {
                    Valid::from_iter(rhs, |(key, value)| {
                        if let Some(lhs_value) = lhs.remove(&key) {
                            value.merge_right(lhs_value).map(|value| (key, value))
                        } else {
                            Valid::succeed((key, value))
                        }
                    }).and_then(|(value)| {
                        for (k,v) in value {
                            lhs.insert(k,v);
                        }
                        Valid::succeed(Value::Mapping(lhs))
                    })
                }
                Value::Sequence(mut rhs) => {
                    rhs.push(Value::Mapping(lhs));
                    Valid::succeed(Value::Sequence(rhs))
                }
                other => Valid::succeed(other),
            },
            (Value::Tagged(mut lhs), other) => match other {
                Value::Tagged(rhs) => {
                    if lhs.tag == rhs.tag {
                        lhs.value.clone().merge_right(rhs.value).and_then(|value| {
                            lhs.value = value;
                            Valid::succeed(Value::Tagged(lhs))
                        })
                    } else {
                        Valid::succeed(Value::Tagged(rhs))
                    }
                }
                other => Valid::succeed(other),
            },
        }
    }
}
