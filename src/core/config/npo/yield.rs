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

impl<'a> Yield<'a> {
    pub fn as_vec(&self) -> Vec<Vec<(&'a str, (&'a str, &'a str))>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();

        #[allow(clippy::too_many_arguments)]
        fn dfs<'a>(
            map: &HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
            ty: TypeName<'a>,
            path: Vec<(&'a str, (&'a str, &'a str))>,
            result: &mut Vec<Vec<(&'a str, (&'a str, &'a str))>>,
            visited: &mut HashSet<(TypeName<'a>, FieldName<'a>)>,
        ) {
            if let Some(fields) = map.get(&ty) {
                for (field_name, ty_of) in fields {
                    let mut new_path = path.clone();
                    new_path.push((ty.0, (field_name.0, ty_of.0)));
                    if !visited.contains(&(ty, *field_name)) {
                        visited.insert((ty, *field_name));
                        dfs(map, *ty_of, new_path, result, visited);
                        visited.remove(&(ty, *field_name));
                    }
                }
            } else {
                result.push(path);
            }
        }

        dfs(
            &self.map,
            TypeName(self.root),
            Vec::new(),
            &mut result,
            &mut visited,
        );

        result
    }
}
