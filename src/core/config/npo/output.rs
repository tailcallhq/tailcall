use std::collections::{HashMap, HashSet};

use derive_getters::Getters;

use crate::core::config::npo::{FieldName, TypeName};

#[derive(Default, Debug, PartialEq, Getters)]
pub struct Output<'a> {
    map: HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
    root: &'a str,
}

impl<'a> Output<'a> {
    pub fn new(
        map: HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
        root: &'a str,
    ) -> Self {
        Self { map, root }
    }
    pub fn query_paths(&self) -> Vec<Vec<FieldName<'a>>> {
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
                    new_path.push((ty.0, (field_name.as_str(), ty_of.0)));
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
            .into_iter()
            .map(|v| v.into_iter().map(|(_, (f, _))| FieldName::new(f)).collect())
            .collect()
    }
}
