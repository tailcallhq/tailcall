use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::ops::Deref;

use crate::core::config::Config;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TypeName<'a>(pub &'a str);
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct FieldName<'a>(pub &'a str);

impl Display for TypeName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for FieldName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Default, Debug, PartialEq)]
struct YieldInner<'a>(HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>);

impl<'a> YieldInner<'a> {
    fn into_yield(self, root: &'a str) -> Yield<'a> {
        Yield { map: self.0, root }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Yield<'a> {
    pub map: HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>,
    pub root: &'a str,
}

impl<'a> Deref for Yield<'a> {
    type Target = HashMap<TypeName<'a>, HashSet<(FieldName<'a>, TypeName<'a>)>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

struct FindFanOutContext<'a> {
    config: &'a Config,
    type_name: &'a str,
    is_list: bool,
}

#[inline(always)]
fn find_fan_out<'a>(
    ctx: FindFanOutContext<'a>,
    visited: &mut HashMap<TypeName<'a>, HashSet<FieldName<'a>>>,
) -> YieldInner<'a> {
    let config = ctx.config;
    let type_name: TypeName = TypeName(ctx.type_name);
    let is_list = ctx.is_list;
    let mut ans = HashMap::new();

    if let Some(type_) = config.find_type(type_name.0) {
        for (field_name, field) in type_.fields.iter() {
            let cur: FieldName = FieldName(field_name.as_str());
            let tuple: (FieldName, TypeName) = (cur, TypeName(field.type_of.as_str()));

            let condition = visited
                .get(&type_name)
                .map(|v: &HashSet<FieldName>| v.contains(&cur))
                .unwrap_or_default();

            if condition {
                continue;
            } else {
                visited.entry(type_name).or_default().insert(cur);
            }

            if field.has_resolver() && !field.has_batched_resolver() && is_list {
                ans.entry(type_name).or_insert(HashSet::new()).insert(tuple);
            } else {
                let next = find_fan_out(
                    FindFanOutContext {
                        config,
                        type_name: &field.type_of,
                        is_list: field.list || is_list,
                    },
                    visited,
                );
                for (k, v) in next.0 {
                    ans.entry(k).or_insert(HashSet::new()).extend(v);
                    ans.entry(type_name).or_insert(HashSet::new()).insert(tuple);
                }
            }
        }
    }

    YieldInner(ans)
}

pub fn n_plus_one(config: &Config) -> Yield {
    let mut visited = HashMap::new();
    if let Some(query) = &config.schema.query {
        let yield_inner = find_fan_out(
            FindFanOutContext { config, type_name: query, is_list: false },
            &mut visited,
        );
        yield_inner.into_yield(query)
    } else {
        Default::default()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use crate::core::config::npo::Yield;
    use crate::core::config::{Config, Field, Http, Type};

    macro_rules! assert_eq_map {
        ($actual:expr, $expected_vec:expr) => {{
            let mut expected: HashMap<TypeName, HashSet<(FieldName, TypeName)>> = HashMap::new();

            for vec in $expected_vec {
                for value in vec {
                    let (key, (value, ty_of)) = value;
                    let key = TypeName(key);
                    let value = (FieldName(value), TypeName(ty_of));

                    expected
                        .entry(key)
                        .or_insert_with(HashSet::new)
                        .insert(value);
                }
            }

            assert_eq!($actual, Yield { map: expected, root: ($actual).root });
        }};
    }

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
        let expected = vec![vec![("Query", ("f1", "F1")), ("F1", ("f2", "F2"))]];

        assert_eq_map!(actual, expected);
    }

    #[test]
    fn test_nplusone_batched_resolvers() {
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
                        .http(Http { batch_key: vec!["id".into()], ..Default::default() }),
                )]),
            ),
            (
                "F2",
                Type::default()
                    .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected: Vec<Vec<_>> = vec![];
        assert_eq_map!(actual, expected);
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
        let expected = vec![vec![
            ("Query", ("f1", "F1")),
            ("F1", ("f2", "F2")),
            ("F2", ("f3", "F3")),
            ("F3", ("f4", "String")),
        ]];

        assert_eq_map!(actual, expected);
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

        let expected = vec![vec![
            ("Query", ("f1", "F1")),
            ("F1", ("f2", "F2")),
            ("F2", ("f3", "F3")),
            ("F3", ("f4", "String")),
        ]];
        let actual = config.n_plus_one();

        assert_eq_map!(actual, expected);
    }

    #[test]
    fn test_nplusone_nested_resolvers_without_resolvers() {
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
                Type::default()
                    .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
            ),
        ]);

        let expected: Vec<Vec<_>> = vec![];
        let actual = config.n_plus_one();

        assert_eq_map!(actual, expected);
    }

    #[test]
    fn test_nplusone_cycles() {
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
                    ("f1", Field::default().type_of("F1".to_string())),
                    ("f2", Field::default().type_of("F2".to_string()).into_list()),
                ]),
            ),
            (
                "F2",
                Type::default()
                    .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected: Vec<Vec<_>> = vec![];

        assert_eq_map!(actual, expected);
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
        let expected = vec![
            vec![
                ("Query", ("f1", "F1")),
                ("F1", ("f1", "F1")),
                ("F1", ("f2", "String")),
            ],
            vec![("Query", ("f1", "F1")), ("F1", ("f2", "String"))],
        ];

        assert_eq_map!(actual, expected);
    }
    #[test]
    fn test_nplusone_nested_non_list() {
        let f_field = Field::default()
            .type_of("F".to_string())
            .http(Http::default());

        let config = Config::default().query("Query").types(vec![
            ("Query", Type::default().fields(vec![("f", f_field)])),
            (
                "F",
                Type::default().fields(vec![(
                    "g",
                    Field::default()
                        .type_of("G".to_string())
                        .into_list()
                        .http(Http::default()),
                )]),
            ),
            (
                "G",
                Type::default().fields(vec![("e", Field::default().type_of("String".to_string()))]),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected = Vec::<Vec<_>>::new();

        assert_eq_map!(actual, expected);
    }
}
