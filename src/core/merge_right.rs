use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use serde_yaml::Value;

pub trait MergeRight {
    fn merge_right(self, other: Self) -> Self;
}

impl<A: MergeRight> MergeRight for Option<A> {
    fn merge_right(self, other: Self) -> Self {
        match (self, other) {
            (Some(this), Some(that)) => Some(this.merge_right(that)),
            (None, Some(that)) => Some(that),
            (Some(this), None) => Some(this),
            (None, None) => None,
        }
    }
}

impl<A: MergeRight + Default> MergeRight for Arc<A> {
    fn merge_right(self, other: Self) -> Self {
        let l = Arc::into_inner(self);
        let r = Arc::into_inner(other);
        Arc::new(l.merge_right(r).unwrap_or_default())
    }
}

impl<A> MergeRight for Vec<A> {
    fn merge_right(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

impl<K, V> MergeRight for BTreeMap<K, V>
where
    K: Ord,
    V: Clone + MergeRight,
{
    fn merge_right(mut self, other: Self) -> Self {
        for (other_name, mut other_value) in other {
            if let Some(self_value) = self.remove(&other_name) {
                other_value = self_value.merge_right(other_value);
            }

            self.insert(other_name, other_value);
        }
        self
    }
}

impl<V> MergeRight for BTreeSet<V>
where
    V: Ord,
{
    fn merge_right(mut self, other: Self) -> Self {
        self.extend(other);
        self
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
    fn merge_right(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

impl MergeRight for Value {
    fn merge_right(self, other: Self) -> Self {
        match (self, other) {
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
                    for (key, mut value) in rhs {
                        if let Some(lhs_value) = lhs.remove(&key) {
                            value = lhs_value.merge_right(value);
                        }
                        lhs.insert(key, value);
                    }
                    Value::Mapping(lhs)
                }
                Value::Sequence(mut rhs) => {
                    rhs.push(Value::Mapping(lhs));
                    Value::Sequence(rhs)
                }
                other => other,
            },
            (Value::Tagged(mut lhs), other) => match other {
                Value::Tagged(rhs) => {
                    if lhs.tag == rhs.tag {
                        lhs.value = lhs.value.merge_right(rhs.value);
                        Value::Tagged(lhs)
                    } else {
                        Value::Tagged(rhs)
                    }
                }
                other => other,
            },
        }
    }
}
