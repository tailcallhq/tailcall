use std::marker::PhantomData;

use crate::core::{config::Type, merge_right::MergeRight, valid::Valid};

#[derive(Default)]
pub(super) struct TypeMerger<T>(PhantomData<T>);

pub(super) struct UnionMerge;
pub(super) struct IntersectionMerge;

impl TypeMerger<UnionMerge> {
    pub fn merge(&self, left: Type, right: Type) -> Valid<Type, String> {
        // let mut fields = left.fields;

        // for (name, mut rfield) in right.fields {
        //     if let Some(lfield) = fields.remove(&name) {
        //         rfield = lfield.merge_right(rfield);
        //     }

        //     fields.insert(name, rfield);
        // }

        Valid::succeed(left)
    }
}
