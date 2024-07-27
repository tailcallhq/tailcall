use std::collections::{HashMap, HashSet};

use colored::*;

use crate::core::config::{Config, FieldName, TypeName};

pub struct Fmt {}

impl Fmt {
    pub fn heading(heading: &str) -> String {
        format!("{}", heading.bold())
    }

    pub fn meta(meta: &String) -> String {
        format!("{}", meta.yellow())
    }

    pub fn display(s: String) {
        println!("{}", s);
    }

    fn reduce<'a>(
        map: &HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
        root: TypeName<'a>,
    ) -> Vec<Vec<(&'a str, (&'a str, &'a str))>> {
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

        dfs(map, root, Vec::new(), &mut result, &mut visited);

        result
    }

    pub fn format_n_plus_one_queries(
        n_plus_one_info: HashMap<TypeName, HashSet<(FieldName, TypeName)>>,
        root: &str,
    ) -> String {
        let reduced = Fmt::reduce(&n_plus_one_info, TypeName(root));

        let query_paths: Vec<Vec<_>> = reduced
            .iter()
            .map(|item| {
                item.iter()
                    .map(|(_, (field_name, _))| *field_name)
                    .collect()
            })
            .collect();

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

        Fmt::meta(&query_data.iter().rfold("".to_string(), |s, query| {
            if s.is_empty() {
                query.to_string()
            } else {
                format!("{}\n{}", query, s)
            }
        }))
    }

    pub fn log_n_plus_one(show_npo: bool, config: &Config) {
        let n_plus_one_info = config.n_plus_one();
        let mut message = format!("N + 1 detected: {}", n_plus_one_info.len());

        if show_npo {
            message.push('\n');
            message.push_str(&Fmt::format_n_plus_one_queries(
                n_plus_one_info,
                config.schema.query.as_deref().unwrap_or_default(),
            ));
        }

        tracing::info!("{}", message);
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::fmt::Fmt;

    use crate::core::config::{Config, Field, Http, Type};
    #[test]
    fn test_nplusone_resolvers() {
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
        let formatted = Fmt::format_n_plus_one_queries(
            actual,
            config.schema.query.as_deref().unwrap_or_default(),
        );

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn test_nplusone_nested_resolvers() {
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
        let formatted = Fmt::format_n_plus_one_queries(
            actual,
            config.schema.query.as_deref().unwrap_or_default(),
        );

        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn test_nplusone_nested_resolvers_non_list_resolvers() {
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
        let formatted = Fmt::format_n_plus_one_queries(
            actual,
            config.schema.query.as_deref().unwrap_or_default(),
        );
        insta::assert_snapshot!(formatted);
    }

    #[test]
    fn test_nplusone_cycles_with_resolvers() {
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

        let formatted = Fmt::format_n_plus_one_queries(
            actual,
            config.schema.query.as_deref().unwrap_or_default(),
        );
        insta::assert_snapshot!(formatted);
    }
}
