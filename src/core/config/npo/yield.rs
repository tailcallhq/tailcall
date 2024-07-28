use std::collections::{HashMap, HashSet};

use crate::core::config::npo::{FieldName, TypeName};

#[derive(Default, Debug, PartialEq)]
pub(super) struct YieldInner<'a>(
    pub(super) HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
);

impl<'a> YieldInner<'a> {
    pub fn into_yield(self, root: &'a str) -> Yield<'a> {
        Yield { map: self.0, root }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Yield<'a> {
    pub map: HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
    pub root: &'a str,
}
