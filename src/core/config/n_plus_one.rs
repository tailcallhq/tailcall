use crate::core::config::Config;

struct FindFanOutContext<'a> {
    config: &'a Config,
    type_name: &'a String,
    path: Vec<(String, String)>,
    is_list: bool,
}

fn find_fan_out(context: FindFanOutContext) -> Vec<Vec<(String, String)>> {
    let config = context.config;
    let type_name = context.type_name;
    let path = context.path;
    let is_list = context.is_list;
    match config.find_type(type_name) {
        Some(type_) => type_
            .fields
            .iter()
            .flat_map(|(field_name, field)| {
                let mut new_path = path.clone();
                new_path.push((type_name.clone(), field_name.clone()));
                if path
                    .iter()
                    .any(|item| &item.0 == type_name && &item.1 == field_name)
                {
                    Vec::new()
                } else if field.has_resolver() && !field.has_batched_resolver() && is_list {
                    vec![new_path]
                } else {
                    find_fan_out(FindFanOutContext {
                        config,
                        type_name: &field.type_of,
                        path: new_path,
                        is_list: field.list || is_list,
                    })
                }
            })
            .collect(),
        None => Vec::new(),
    }
}

pub fn n_plus_one(config: &Config) -> Vec<Vec<(String, String)>> {
    if let Some(query) = &config.schema.query {
        find_fan_out(FindFanOutContext {
            config,
            type_name: query,
            path: Vec::new(),
            is_list: false,
        })
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {

    use crate::core::config::{Config, Field, Http, Type};

    use crate::core::config::position::Pos;

    #[test]
    fn test_nplusone_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f1",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default()
                                .type_of("F1".to_string())
                                .into_list()
                                .http(Pos::new(0, 0, None, Http::default())),
                        ),
                    )]),
                ),
            ),
            (
                "F1",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f2",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default()
                                .type_of("F2".to_string())
                                .into_list()
                                .http(Pos::new(0, 0, None, Http::default())),
                        ),
                    )]),
                ),
            ),
            (
                "F2",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f3",
                        Pos::new(0, 0, None, Field::default().type_of("String".to_string())),
                    )]),
                ),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected = vec![vec![
            ("Query".to_string(), "f1".to_string()),
            ("F1".to_string(), "f2".to_string()),
        ]];
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_nplusone_batched_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f1",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default()
                                .type_of("F1".to_string())
                                .into_list()
                                .http(Pos::new(0, 0, None, Http::default())),
                        ),
                    )]),
                ),
            ),
            (
                "F1",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f2",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default()
                                .type_of("F2".to_string())
                                .into_list()
                                .http(Pos::new(
                                    0,
                                    0,
                                    None,
                                    Http {
                                        group_by: Pos::new(0, 0, None, vec!["id".into()]),
                                        ..Default::default()
                                    },
                                )),
                        ),
                    )]),
                ),
            ),
            (
                "F2",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f3",
                        Pos::new(0, 0, None, Field::default().type_of("String".to_string())),
                    )]),
                ),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected: Vec<Vec<(String, String)>> = vec![];
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_nplusone_nested_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f1",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default()
                                .type_of("F1".to_string())
                                .into_list()
                                .http(Pos::new(0, 0, None, Http::default())),
                        ),
                    )]),
                ),
            ),
            (
                "F1",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f2",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default().type_of("F2".to_string()).into_list(),
                        ),
                    )]),
                ),
            ),
            (
                "F2",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f3",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default().type_of("F3".to_string()).into_list(),
                        ),
                    )]),
                ),
            ),
            (
                "F3",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f4",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default()
                                .type_of("String".to_string())
                                .http(Pos::new(0, 0, None, Http::default())),
                        ),
                    )]),
                ),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected = vec![vec![
            ("Query".to_string(), "f1".to_string()),
            ("F1".to_string(), "f2".to_string()),
            ("F2".to_string(), "f3".to_string()),
            ("F3".to_string(), "f4".to_string()),
        ]];
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_nplusone_nested_resolvers_non_list_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f1",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default().type_of("F1".to_string()).http(Pos::new(
                                0,
                                0,
                                None,
                                Http::default(),
                            )),
                        ),
                    )]),
                ),
            ),
            (
                "F1",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f2",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default().type_of("F2".to_string()).into_list(),
                        ),
                    )]),
                ),
            ),
            (
                "F2",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f3",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default().type_of("F3".to_string()).into_list(),
                        ),
                    )]),
                ),
            ),
            (
                "F3",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f4",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default()
                                .type_of("String".to_string())
                                .http(Pos::new(0, 0, None, Http::default())),
                        ),
                    )]),
                ),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected = vec![vec![
            ("Query".to_string(), "f1".to_string()),
            ("F1".to_string(), "f2".to_string()),
            ("F2".to_string(), "f3".to_string()),
            ("F3".to_string(), "f4".to_string()),
        ]];
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_nplusone_nested_resolvers_without_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f1",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default()
                                .type_of("F1".to_string())
                                .into_list()
                                .http(Pos::new(0, 0, None, Http::default())),
                        ),
                    )]),
                ),
            ),
            (
                "F1",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f2",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default().type_of("F2".to_string()).into_list(),
                        ),
                    )]),
                ),
            ),
            (
                "F2",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f3",
                        Pos::new(0, 0, None, Field::default().type_of("String".to_string())),
                    )]),
                ),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected: Vec<Vec<(String, String)>> = vec![];
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_nplusone_cycles() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f1",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default()
                                .type_of("F1".to_string())
                                .into_list()
                                .http(Pos::new(0, 0, None, Http::default())),
                        ),
                    )]),
                ),
            ),
            (
                "F1",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![
                        (
                            "f1",
                            Pos::new(0, 0, None, Field::default().type_of("F1".to_string())),
                        ),
                        (
                            "f2",
                            Pos::new(
                                0,
                                0,
                                None,
                                Field::default().type_of("F2".to_string()).into_list(),
                            ),
                        ),
                    ]),
                ),
            ),
            (
                "F2",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f3",
                        Pos::new(0, 0, None, Field::default().type_of("String".to_string())),
                    )]),
                ),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected: Vec<Vec<(String, String)>> = vec![];
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_nplusone_cycles_with_resolvers() {
        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f1",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default()
                                .type_of("F1".to_string())
                                .into_list()
                                .http(Pos::new(0, 0, None, Http::default())),
                        ),
                    )]),
                ),
            ),
            (
                "F1",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![
                        (
                            "f1",
                            Pos::new(
                                0,
                                0,
                                None,
                                Field::default().type_of("F1".to_string()).into_list(),
                            ),
                        ),
                        (
                            "f2",
                            Pos::new(
                                0,
                                0,
                                None,
                                Field::default()
                                    .type_of("String".to_string())
                                    .http(Pos::new(0, 0, None, Http::default())),
                            ),
                        ),
                    ]),
                ),
            ),
            (
                "F2",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "f3",
                        Pos::new(0, 0, None, Field::default().type_of("String".to_string())),
                    )]),
                ),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected = vec![
            vec![
                ("Query".to_string(), "f1".to_string()),
                ("F1".to_string(), "f1".to_string()),
                ("F1".to_string(), "f2".to_string()),
            ],
            vec![
                ("Query".to_string(), "f1".to_string()),
                ("F1".to_string(), "f2".to_string()),
            ],
        ];

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_nplusone_nested_non_list() {
        let f_field = Pos::new(
            0,
            0,
            None,
            Field::default()
                .type_of("F".to_string())
                .http(Pos::new(0, 0, None, Http::default())),
        );

        let config = Config::default().query("Query").types(vec![
            (
                "Query",
                Pos::new(0, 0, None, Type::default().fields(vec![("f", f_field)])),
            ),
            (
                "F",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "g",
                        Pos::new(
                            0,
                            0,
                            None,
                            Field::default()
                                .type_of("G".to_string())
                                .into_list()
                                .http(Pos::new(0, 0, None, Http::default())),
                        ),
                    )]),
                ),
            ),
            (
                "G",
                Pos::new(
                    0,
                    0,
                    None,
                    Type::default().fields(vec![(
                        "e",
                        Pos::new(0, 0, None, Field::default().type_of("String".to_string())),
                    )]),
                ),
            ),
        ]);

        let actual = config.n_plus_one();
        let expected = Vec::<Vec<(String, String)>>::new();

        assert_eq!(actual, expected)
    }
}
