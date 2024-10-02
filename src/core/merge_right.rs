use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use prost_reflect::prost_types::FileDescriptorProto;

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

impl<A> MergeRight for Vec<A> {
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

impl<K, V> MergeRight for BTreeMap<K, V>
where
    K: Ord,
    V: MergeRight,
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

impl<K, V> MergeRight for HashMap<K, V>
where
    K: Eq + std::hash::Hash,
    V: MergeRight,
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

impl MergeRight for FileDescriptorProto {
    fn merge_right(self, other: Self) -> Self {
        other
    }
}

impl MergeRight for serde_yaml::Value {
    fn merge_right(self, other: Self) -> Self {
        use serde_yaml::Value;

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

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

    use super::MergeRight;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct Test(u32);

    impl From<u32> for Test {
        fn from(value: u32) -> Self {
            Self(value)
        }
    }

    impl MergeRight for Test {
        fn merge_right(self, other: Self) -> Self {
            Self(self.0 + other.0)
        }
    }

    #[test]
    fn test_option() {
        let x: Option<Test> = None.merge_right(None);
        assert_eq!(x, None);

        let x = Some(Test::from(1)).merge_right(None);
        assert_eq!(x, Some(Test::from(1)));

        let x = None.merge_right(Some(Test::from(2)));
        assert_eq!(x, Some(Test::from(2)));

        let x = Some(Test::from(1)).merge_right(Some(Test::from(2)));
        assert_eq!(x, Some(Test::from(3)));
    }

    #[test]
    fn test_vec() {
        let l: Vec<Test> = vec![];
        let r: Vec<Test> = vec![];
        assert_eq!(l.merge_right(r), vec![]);

        let l: Vec<Test> = vec![Test::from(1), Test::from(2)];
        let r: Vec<Test> = vec![];
        assert_eq!(l.merge_right(r), vec![Test::from(1), Test::from(2)]);

        let l: Vec<Test> = vec![];
        let r: Vec<Test> = vec![Test::from(3), Test::from(4)];
        assert_eq!(l.merge_right(r), vec![Test::from(3), Test::from(4)]);

        let l: Vec<Test> = vec![Test::from(1), Test::from(2)];
        let r: Vec<Test> = vec![Test::from(3), Test::from(4)];
        assert_eq!(
            l.merge_right(r),
            vec![Test::from(1), Test::from(2), Test::from(3), Test::from(4)]
        );
    }

    #[test]
    fn test_btree_set() {
        let l: BTreeSet<Test> = BTreeSet::from_iter(vec![]);
        let r: BTreeSet<Test> = BTreeSet::from_iter(vec![]);
        assert_eq!(l.merge_right(r), BTreeSet::from_iter(vec![]));

        let l: BTreeSet<Test> = BTreeSet::from_iter(vec![Test::from(1), Test::from(2)]);
        let r: BTreeSet<Test> = BTreeSet::from_iter(vec![]);
        assert_eq!(
            l.merge_right(r),
            BTreeSet::from_iter(vec![Test::from(1), Test::from(2)])
        );

        let l: BTreeSet<Test> = BTreeSet::from_iter(vec![]);
        let r: BTreeSet<Test> = BTreeSet::from_iter(vec![Test::from(3), Test::from(4)]);
        assert_eq!(
            l.merge_right(r),
            BTreeSet::from_iter(vec![Test::from(3), Test::from(4)])
        );

        let l: BTreeSet<Test> = BTreeSet::from_iter(vec![Test::from(1), Test::from(2)]);
        let r: BTreeSet<Test> =
            BTreeSet::from_iter(vec![Test::from(2), Test::from(3), Test::from(4)]);
        assert_eq!(
            l.merge_right(r),
            BTreeSet::from_iter(vec![
                Test::from(1),
                Test::from(2),
                Test::from(3),
                Test::from(4)
            ])
        );
    }

    #[test]
    fn test_hash_set() {
        let l: HashSet<Test> = HashSet::from_iter(vec![]);
        let r: HashSet<Test> = HashSet::from_iter(vec![]);
        assert_eq!(l.merge_right(r), HashSet::from_iter(vec![]));

        let l: HashSet<Test> = HashSet::from_iter(vec![Test::from(1), Test::from(2)]);
        let r: HashSet<Test> = HashSet::from_iter(vec![]);
        assert_eq!(
            l.merge_right(r),
            HashSet::from_iter(vec![Test::from(1), Test::from(2)])
        );

        let l: HashSet<Test> = HashSet::from_iter(vec![]);
        let r: HashSet<Test> = HashSet::from_iter(vec![Test::from(3), Test::from(4)]);
        assert_eq!(
            l.merge_right(r),
            HashSet::from_iter(vec![Test::from(3), Test::from(4)])
        );

        let l: HashSet<Test> = HashSet::from_iter(vec![Test::from(1), Test::from(2)]);
        let r: HashSet<Test> =
            HashSet::from_iter(vec![Test::from(2), Test::from(3), Test::from(4)]);
        assert_eq!(
            l.merge_right(r),
            HashSet::from_iter(vec![
                Test::from(1),
                Test::from(2),
                Test::from(3),
                Test::from(4)
            ])
        );
    }

    #[test]
    fn test_btree_map() {
        let l: BTreeMap<u32, Test> = BTreeMap::from_iter(vec![]);
        let r: BTreeMap<u32, Test> = BTreeMap::from_iter(vec![]);
        assert_eq!(l.merge_right(r), BTreeMap::from_iter(vec![]));

        let l: BTreeMap<u32, Test> =
            BTreeMap::from_iter(vec![(1, Test::from(1)), (2, Test::from(2))]);
        let r: BTreeMap<u32, Test> = BTreeMap::from_iter(vec![]);
        assert_eq!(
            l.merge_right(r),
            BTreeMap::from_iter(vec![(1, Test::from(1)), (2, Test::from(2))])
        );

        let l: BTreeMap<u32, Test> = BTreeMap::from_iter(vec![]);
        let r: BTreeMap<u32, Test> =
            BTreeMap::from_iter(vec![(3, Test::from(3)), (4, Test::from(4))]);
        assert_eq!(
            l.merge_right(r),
            BTreeMap::from_iter(vec![(3, Test::from(3)), (4, Test::from(4))])
        );

        let l: BTreeMap<u32, Test> =
            BTreeMap::from_iter(vec![(1, Test::from(1)), (2, Test::from(2))]);
        let r: BTreeMap<u32, Test> = BTreeMap::from_iter(vec![
            (2, Test::from(5)),
            (3, Test::from(3)),
            (4, Test::from(4)),
        ]);
        assert_eq!(
            l.merge_right(r),
            BTreeMap::from_iter(vec![
                (1, Test::from(1)),
                (2, Test::from(7)),
                (3, Test::from(3)),
                (4, Test::from(4))
            ])
        );
    }

    #[test]
    fn test_hash_map() {
        let l: HashMap<u32, Test> = HashMap::from_iter(vec![]);
        let r: HashMap<u32, Test> = HashMap::from_iter(vec![]);
        assert_eq!(l.merge_right(r), HashMap::from_iter(vec![]));

        let l: HashMap<u32, Test> =
            HashMap::from_iter(vec![(1, Test::from(1)), (2, Test::from(2))]);
        let r: HashMap<u32, Test> = HashMap::from_iter(vec![]);
        assert_eq!(
            l.merge_right(r),
            HashMap::from_iter(vec![(1, Test::from(1)), (2, Test::from(2))])
        );

        let l: HashMap<u32, Test> = HashMap::from_iter(vec![]);
        let r: HashMap<u32, Test> =
            HashMap::from_iter(vec![(3, Test::from(3)), (4, Test::from(4))]);
        assert_eq!(
            l.merge_right(r),
            HashMap::from_iter(vec![(3, Test::from(3)), (4, Test::from(4))])
        );

        let l: HashMap<u32, Test> =
            HashMap::from_iter(vec![(1, Test::from(1)), (2, Test::from(2))]);
        let r: HashMap<u32, Test> = HashMap::from_iter(vec![
            (2, Test::from(5)),
            (3, Test::from(3)),
            (4, Test::from(4)),
        ]);
        assert_eq!(
            l.merge_right(r),
            HashMap::from_iter(vec![
                (1, Test::from(1)),
                (2, Test::from(7)),
                (3, Test::from(3)),
                (4, Test::from(4))
            ])
        );
    }
}
