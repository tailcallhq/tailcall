use std::collections::{HashMap, HashSet};

use crate::core::config::Config;

struct FindFanOutContext1<'a> {
    config: &'a Config,
    type_name: &'a str,
    is_list: bool,
}

#[inline(always)]
fn find_fan_out1<'a>(
    ctx: FindFanOutContext1<'a>,
    visited: &mut HashMap<&'a str, HashSet<&'a str>>,
) -> HashMap<&'a str, HashSet<&'a str>> {
    let config = ctx.config;
    let type_name = ctx.type_name;
    let is_list = ctx.is_list;
    let mut ans = HashMap::new();

    if let Some(type_) = config.find_type(type_name) {
        for (field_name, field) in type_.fields.iter() {
            let cur = field_name.as_str();

            let x = visited
                .get(type_name)
                .map(|v: &HashSet<&str>| v.contains(cur))
                .unwrap_or_default();
            if x {
                continue;
            } else {
                visited
                    .entry(type_name)
                    .or_default()
                    .insert(cur);
            }

            if field.has_resolver() && !field.has_batched_resolver() && is_list {
                ans.entry(type_name).or_insert(HashSet::new()).insert(cur);
            } else {
                let next = find_fan_out1(
                    FindFanOutContext1 {
                        config,
                        type_name: &field.type_of,
                        is_list: field.list || is_list,
                    },
                    visited,
                );
                for (k, v) in next {
                    ans.entry(k).or_insert(HashSet::new()).extend(v);
                    if let Some(set) = ans.get_mut(type_name) {
                        set.insert(cur);
                    } else {
                        let mut set = HashSet::new();
                        set.insert(cur);
                        ans.insert(type_name, set);
                    }
                }
            }
        }
    }

    ans
}

pub fn n_plus_one(config: &Config) -> HashMap<&str, HashSet<&str>> {
    // let mut map = HashMap::new();
    let mut visited = HashMap::new();
    if let Some(query) = &config.schema.query {
        find_fan_out1(
            FindFanOutContext1 { config, type_name: query, is_list: false },
            &mut visited,
        )
    } else {
        Default::default()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::core::config::{Config, Field, Http, Type};

    macro_rules! assert_eq_map {
        ($actual:expr, $expected_vec:expr) => {{
            // Define the conversion logic
            let mut expected: HashMap<&str, HashSet<&str>> = HashMap::new();

            for vec in $expected_vec {
                for value in vec {
                    let (key, value) = value;
                    expected
                        .entry(key)
                        .or_insert_with(HashSet::new)
                        .insert(value);
                }
            }

            // Assert equality
            assert_eq!($actual, expected);
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
        let expected = vec![vec![("Query", "f1"), ("F1", "f2")]];

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
        let expected: Vec<Vec<(&str, &str)>> = vec![];
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
            ("Query", "f1"),
            ("F1", "f2"),
            ("F2", "f3"),
            ("F3", "f4"),
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
            ("Query", "f1"),
            ("F1", "f2"),
            ("F2", "f3"),
            ("F3", "f4"),
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

        let expected: Vec<Vec<(&str, &str)>> = vec![];
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
        let expected: Vec<Vec<(&str, &str)>> = vec![];

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
            vec![("Query", "f1"), ("F1", "f1"), ("F1", "f2")],
            vec![("Query", "f1"), ("F1", "f2")],
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
        let expected = Vec::<Vec<(&str, &str)>>::new();

        assert_eq_map!(actual, expected);
    }
}
