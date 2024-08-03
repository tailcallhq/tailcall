use std::fmt::{Display, Formatter};

use super::{chunk::Chunk, FieldName};

///
/// Represents a list of query paths that can issue a N + 1 query
#[derive(Default, Debug, PartialEq)]
pub struct Queries<'a>(Vec<Vec<&'a str>>);

impl Queries<'_> {
    pub fn size(&self) -> usize {
        self.0.len()
    }
    pub fn from_chunk<'a>(chunk: Chunk<Chunk<FieldName<'a>>>) -> Queries<'a> {
        Queries(
            chunk
                .as_vec()
                .iter()
                .map(|chunk| {
                    chunk
                        .as_vec()
                        .iter()
                        .map(|field_name| field_name.as_str())
                        .collect()
                })
                .collect(),
        )
    }
}

impl<'a> Display for Queries<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let query_data: Vec<String> = self
            .0
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
}
