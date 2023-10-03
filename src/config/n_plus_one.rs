use crate::config::Config;

pub fn n_plus_one(config: &Config) -> Vec<Vec<(String, String)>> {
  #![allow(clippy::too_many_arguments)]
  fn find_fan_out(
    config: &Config,
    type_name: &String,
    path: Vec<(String, String)>,
    is_list: bool,
  ) -> Vec<Vec<(String, String)>> {
    match config.find_type(type_name) {
      Some(type_) => type_
        .fields
        .iter()
        .flat_map(|(field_name, field)| {
          let mut new_path = path.clone();
          new_path.push((type_name.clone(), field_name.clone()));
          if path.iter().any(|item| &item.0 == type_name && &item.1 == field_name) {
            Vec::new()
          } else if field.has_resolver() && !field.has_batched_resolver() && is_list {
            vec![new_path]
          } else {
            find_fan_out(config, &field.type_of, new_path, field.list || is_list)
          }
        })
        .collect(),
      None => Vec::new(),
    }
  }

  if let Some(query) = &config.graphql.schema.query {
    find_fan_out(config, query, Vec::new(), false)
  } else {
    Vec::new()
  }
}

#[cfg(test)]

mod tests {
  use crate::config::{Config, Field, Http, Type};

  #[test]
  fn test_nplusone_resolvers() {
    let config = Config::default().query("Query").types(vec![
      (
        "Query",
        Type::default().fields(vec![(
          "f1",
          Field::default()
            .type_of("F1".to_string())
            .to_list()
            .http(Http::default()),
        )]),
      ),
      (
        "F1",
        Type::default().fields(vec![(
          "f2",
          Field::default()
            .type_of("F2".to_string())
            .to_list()
            .http(Http::default()),
        )]),
      ),
      (
        "F2",
        Type::default().fields(vec![("f3", Field::default().type_of("String".to_string()))]),
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
        Type::default().fields(vec![(
          "f1",
          Field::default()
            .type_of("F1".to_string())
            .to_list()
            .http(Http::default()),
        )]),
      ),
      (
        "F1",
        Type::default().fields(vec![(
          "f2",
          Field::default()
            .type_of("F2".to_string())
            .to_list()
            .http(Http::default().batch_key("b")),
        )]),
      ),
      (
        "F2",
        Type::default().fields(vec![("f3", Field::default().type_of("String".to_string()))]),
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
        Type::default().fields(vec![(
          "f1",
          Field::default()
            .type_of("F1".to_string())
            .to_list()
            .http(Http::default()),
        )]),
      ),
      (
        "F1",
        Type::default().fields(vec![("f2", Field::default().type_of("F2".to_string()).to_list())]),
      ),
      (
        "F2",
        Type::default().fields(vec![("f3", Field::default().type_of("F3".to_string()).to_list())]),
      ),
      (
        "F3",
        Type::default().fields(vec![(
          "f4",
          Field::default().type_of("String".to_string()).http(Http::default()),
        )]),
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
        Type::default().fields(vec![(
          "f1",
          Field::default().type_of("F1".to_string()).http(Http::default()),
        )]),
      ),
      (
        "F1",
        Type::default().fields(vec![("f2", Field::default().type_of("F2".to_string()).to_list())]),
      ),
      (
        "F2",
        Type::default().fields(vec![("f3", Field::default().type_of("F3".to_string()).to_list())]),
      ),
      (
        "F3",
        Type::default().fields(vec![(
          "f4",
          Field::default().type_of("String".to_string()).http(Http::default()),
        )]),
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
        Type::default().fields(vec![(
          "f1",
          Field::default()
            .type_of("F1".to_string())
            .to_list()
            .http(Http::default()),
        )]),
      ),
      (
        "F1",
        Type::default().fields(vec![("f2", Field::default().type_of("F2".to_string()).to_list())]),
      ),
      (
        "F2",
        Type::default().fields(vec![("f3", Field::default().type_of("String".to_string()))]),
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
        Type::default().fields(vec![(
          "f1",
          Field::default()
            .type_of("F1".to_string())
            .to_list()
            .http(Http::default()),
        )]),
      ),
      (
        "F1",
        Type::default().fields(vec![
          ("f1", Field::default().type_of("F1".to_string())),
          ("f2", Field::default().type_of("F2".to_string()).to_list()),
        ]),
      ),
      (
        "F2",
        Type::default().fields(vec![("f3", Field::default().type_of("String".to_string()))]),
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
        Type::default().fields(vec![(
          "f1",
          Field::default()
            .type_of("F1".to_string())
            .to_list()
            .http(Http::default()),
        )]),
      ),
      (
        "F1",
        Type::default().fields(vec![
          ("f1", Field::default().type_of("F1".to_string()).to_list()),
          (
            "f2",
            Field::default().type_of("String".to_string()).http(Http::default()),
          ),
        ]),
      ),
      (
        "F2",
        Type::default().fields(vec![("f3", Field::default().type_of("String".to_string()))]),
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
    let f_field = Field::default().type_of("F".to_string()).http(Http::default());

    let config = Config::default().query("Query").types(vec![
      ("Query", Type::default().fields(vec![("f", f_field)])),
      (
        "F",
        Type::default().fields(vec![(
          "g",
          Field::default()
            .type_of("G".to_string())
            .to_list()
            .http(Http::default()),
        )]),
      ),
      (
        "G",
        Type::default().fields(vec![("e", Field::default().type_of("String".to_string()))]),
      ),
    ]);

    let actual = config.n_plus_one();
    let expected = Vec::<Vec<(String, String)>>::new();

    assert_eq!(actual, expected)
  }
}
