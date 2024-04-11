use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::config::{HttpVersion, Proxy, ScriptOptions};

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

pub trait Scalar {}

impl Scalar for u64 {}

impl Scalar for u32 {}

impl Scalar for u16 {}

impl Scalar for u8 {}

impl Scalar for usize {}

impl Scalar for i64 {}

impl Scalar for i32 {}

impl Scalar for i16 {}

impl Scalar for i8 {}

impl Scalar for f64 {}

impl Scalar for f32 {}

impl Scalar for bool {}

impl Scalar for char {}

impl Scalar for String {}

impl Scalar for HttpVersion {}

impl MergeRight for ScriptOptions {
    fn merge_right(self, other: Self) -> Self {
        ScriptOptions {
            timeout: self.timeout.merge_right(other.timeout),
        }
    }
}

impl Scalar for Proxy {}

impl<A: Scalar> MergeRight for A {
    fn merge_right(self, other: Self) -> Self {
        other
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
