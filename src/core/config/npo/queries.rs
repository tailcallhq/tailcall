use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

use derive_getters::Getters;

use crate::core::config::npo::{FieldName, TypeName};

///
/// Represents a list of query paths that can issue a N + 1 query
#[derive(Default, Debug, PartialEq, Getters)]
pub struct Queries<'a> {
    map: HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
    root: &'a str,
}

impl<'a> Queries<'a> {
    pub fn new(
        map: HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
        root: &'a str,
    ) -> Self {
        Self { map, root }
    }

    ///
    /// Returns the query paths that can issue a N + 1 query
    pub fn as_path(&self) -> Vec<Vec<FieldName<'a>>> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();

        #[allow(clippy::too_many_arguments)]
        fn dfs<'a>(
            map: &HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
            ty: TypeName<'a>,
            mut path: Vec<(&'a str, (&'a str, &'a str))>,
            result: &mut Vec<Vec<(&'a str, (&'a str, &'a str))>>,
            visited: &mut HashSet<(TypeName<'a>, FieldName<'a>)>,
            leaf: bool,
        ) {
            if leaf {
                path.pop();
                result.push(path);
                return;
            }

            if let Some(fields) = map.get(&ty) {
                for (field_name, ty_of) in fields {
                    let mut new_path = path.clone();
                    new_path.push((ty.as_str(), (field_name.as_str(), ty_of.as_str())));
                    if !visited.contains(&(ty, *field_name)) {
                        visited.insert((ty, *field_name));
                        dfs(map, *ty_of, new_path, result, visited, ty.leaf());
                        visited.remove(&(ty, *field_name));
                    }
                }
            } else {
                result.push(path);
            }
        }

        let root = TypeName::new(self.root);
        let leaf = root.leaf();
        dfs(&self.map, root, Vec::new(), &mut result, &mut visited, leaf);

        result
            .into_iter()
            .map(|v| v.into_iter().map(|(_, (f, _))| FieldName::new(f)).collect())
            .collect()
    }
}

impl<'a> Display for Queries<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let query_paths = self.as_path();

        let query_data: Vec<String> = query_paths
            .iter()
            .map(|query_path| {
                let mut path = "query { ".to_string();
                path.push_str(
                    query_path
                        .iter()
                        .rfold("".to_string(), |s, field_name| {
                            if s.is_empty() {
                                field_name.to_string()
                            } else {
                                format!("{} {{ {} }}", field_name, s)
                            }
                        })
                        .as_str(),
                );
                path.push_str(" }");
                path
            })
            .collect();

        let val = query_data.iter().rfold("".to_string(), |s, query| {
            if s.is_empty() {
                query.to_string()
            } else {
                format!("{}\n{}", query, s)
            }
        });

        f.write_str(&val)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::config::{Config, Field, Http, Type};
    use crate::core::valid::Validator;

    #[test]
    fn test_npo_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Type::default().fields(vec![(
                    "f1",
                    Field::default()
                        .type_of("F1".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "F1",
                Type::default().fields(vec![(
                    "f2",
                    Field::default()
                        .type_of("F2".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "F2",
                Type::default()
                    .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
            ),
        ]);
        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();
        insta::assert_snapshot!(formatted.join("\n"));
    }

    #[test]
    fn test_npo_nested_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Type::default().fields(vec![(
                    "f1",
                    Field::default()
                        .type_of("F1".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "F1",
                Type::default().fields(vec![(
                    "f2",
                    Field::default().type_of("F2".to_string()).into_list(),
                )]),
            ),
            (
                "F2",
                Type::default().fields(vec![(
                    "f3",
                    Field::default().type_of("F3".to_string()).into_list(),
                )]),
            ),
            (
                "F3",
                Type::default().fields(vec![(
                    "f4",
                    Field::default()
                        .type_of("String".to_string())
                        .http(Http::default()),
                )]),
            ),
        ]);

        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();
        insta::assert_snapshot!(formatted.join("\n"));
    }

    #[test]
    fn test_npo_nested_resolvers_non_list_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Type::default().fields(vec![(
                    "f1",
                    Field::default()
                        .type_of("F1".to_string())
                        .http(Http::default()),
                )]),
            ),
            (
                "F1",
                Type::default().fields(vec![(
                    "f2",
                    Field::default().type_of("F2".to_string()).into_list(),
                )]),
            ),
            (
                "F2",
                Type::default().fields(vec![(
                    "f3",
                    Field::default().type_of("F3".to_string()).into_list(),
                )]),
            ),
            (
                "F3",
                Type::default().fields(vec![(
                    "f4",
                    Field::default()
                        .type_of("String".to_string())
                        .http(Http::default()),
                )]),
            ),
        ]);

        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();
        insta::assert_snapshot!(formatted.join("\n"));
    }

    #[test]
    fn test_npo_cycles_with_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Type::default().fields(vec![(
                    "f1",
                    Field::default()
                        .type_of("F1".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "F1",
                Type::default().fields(vec![
                    ("f1", Field::default().type_of("F1".to_string()).into_list()),
                    (
                        "f2",
                        Field::default()
                            .type_of("String".to_string())
                            .http(Http::default()),
                    ),
                ]),
            ),
            (
                "F2",
                Type::default()
                    .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
            ),
        ]);

        let actual = config.n_plus_one();

        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();
        insta::assert_snapshot!(formatted.join("\n"));
    }
    #[test]
    fn test_jp_config() {
        let config = Config::from_sdl(
            std::fs::read_to_string(tailcall_fixtures::configs::JSONPLACEHOLDER_MUTATION)
                .unwrap()
                .as_str(),
        )
        .to_result()
        .unwrap();
        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();
        insta::assert_snapshot!(formatted.join("\n"));
    }
}
