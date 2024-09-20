use crate::core::merge_right::MergeRight;
use crate::core::primitive::Primitive;
use crate::core::valid::Valid;

pub trait FederatedMerge: Sized {
    fn federated_merge(self, other: Self) -> Valid<Self, String>;
}

impl<A: Primitive + Sized> FederatedMerge for A {
    fn federated_merge(self, other: Self) -> Valid<Self, String> {
        Valid::succeed(self.merge_right(other))
    }
}
