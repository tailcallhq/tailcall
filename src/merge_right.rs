use std::collections::{BTreeMap, BTreeSet, HashSet};

pub trait MergeRight {
    fn merge_right(self, other: Self) -> Self;
}

impl<A> MergeRight for Option<A> {
    fn merge_right(mut self, other: Self) -> Self {
        self = other.or(self);
        self
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
    V: Clone,
{
    fn merge_right(mut self, other: Self) -> Self {
        self.extend(other);
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
