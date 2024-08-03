use std::collections::HashSet;
use std::fmt::Display;

use super::chunk::Chunk;
use super::Queries;
use crate::core::config::Config;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TypeName<'a>(pub &'a str);
impl<'a> TypeName<'a> {
    pub fn new(name: &'a str) -> Self {
        Self(name)
    }
    pub fn as_str(self) -> &'a str {
        self.0
    }
}
impl Display for TypeName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct FieldName<'a>(&'a str);
impl<'a> FieldName<'a> {
    pub fn new(name: &'a str) -> Self {
        Self(name)
    }
    pub fn as_str(self) -> &'a str {
        self.0
    }
}
impl Display for FieldName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Identifier<'a> {
    config: &'a Config,
}

impl<'a> Identifier<'a> {
    pub fn new(config: &'a Config) -> Identifier {
        Identifier { config }
    }

    fn iter(
        &self,
        path: Chunk<FieldName<'a>>,
        type_name: TypeName<'a>,
        is_list: bool,
        visited: &mut HashSet<TypeName<'a>>,
    ) -> Chunk<Chunk<FieldName<'a>>> {
        if visited.contains(&type_name) {
            return Chunk::new();
        } else {
            visited.insert(type_name);
        }
        let mut chunks = Chunk::new();
        if let Some(type_of) = self.config.find_type(&type_name.as_str()) {
            for (name, field) in type_of.fields.iter() {
                let path = path.clone().append(FieldName::new(name));
                let is_batch = field.has_batched_resolver();

                if field.has_resolver() {
                    if !is_batch && is_list {
                        chunks = chunks.append(path.clone());
                    }
                }

                let is_list = is_list | field.list;

                chunks = chunks.concat(self.iter(
                    path,
                    TypeName::new(field.type_of.as_str()),
                    is_list,
                    visited,
                ))
            }
        }

        chunks
    }

    fn find_chunks(&self) -> Chunk<Chunk<FieldName<'a>>> {
        match &self.config.schema.query {
            None => Chunk::new(),
            Some(query) => self.iter(
                Chunk::new(),
                TypeName::new(query.as_str()),
                false,
                &mut HashSet::new(),
            ),
        }
    }

    pub fn identify(&self) -> Queries<'a> {
        Queries::from_chunk(self.find_chunks())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::collections::{HashMap, HashSet};

//     use super::*;
//     use crate::core::config::npo::Queries;
//     use crate::core::config::{Config, Field, Http, Type};

//     // macro_rules! assert_eq_map {
//     //     ($actual:expr, $expected_vec:expr) => {{
//     //         let mut expected: HashMap<TypeName, HashSet<(FieldName, TypeName)>> = HashMap::new();

//     //         for vec in $expected_vec {
//     //             for value in vec {
//     //                 let (key, (value, ty_of)) = value;
//     //                 let key = TypeName(key);
//     //                 let value = (FieldName(value), TypeName(ty_of));

//     //                 expected
//     //                     .entry(key)
//     //                     .or_insert_with(HashSet::new)
//     //                     .insert(value);
//     //             }
//     //         }

//     //         assert_eq!($actual, Queries::new(expected, ($actual).root()));
//     //     }};
//     // }

//     #[test]
//     fn test_nplusone_resolvers() {
//         let config = Config::default().query("Query").types(vec![
//             (
//                 "Query",
//                 Type::default().fields(vec![(
//                     "f1",
//                     Field::default()
//                         .type_of("F1".to_string())
//                         .into_list()
//                         .http(Http::default()),
//                 )]),
//             ),
//             (
//                 "F1",
//                 Type::default().fields(vec![(
//                     "f2",
//                     Field::default()
//                         .type_of("F2".to_string())
//                         .into_list()
//                         .http(Http::default()),
//                 )]),
//             ),
//             (
//                 "F2",
//                 Type::default()
//                     .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
//             ),
//         ]);
//         let actual = config.n_plus_one();
//         let expected = vec![vec![("Query", ("f1", "F1")), ("F1", ("f2", "F2"))]];

//         assert_eq_map!(actual, expected);
//     }

//     #[test]
//     fn test_nplusone_batched_resolvers() {
//         let config = Config::default().query("Query").types(vec![
//             (
//                 "Query",
//                 Type::default().fields(vec![(
//                     "f1",
//                     Field::default()
//                         .type_of("F1".to_string())
//                         .into_list()
//                         .http(Http::default()),
//                 )]),
//             ),
//             (
//                 "F1",
//                 Type::default().fields(vec![(
//                     "f2",
//                     Field::default()
//                         .type_of("F2".to_string())
//                         .into_list()
//                         .http(Http { batch_key: vec!["id".into()], ..Default::default() }),
//                 )]),
//             ),
//             (
//                 "F2",
//                 Type::default()
//                     .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
//             ),
//         ]);

//         let actual = config.n_plus_one();
//         let expected: Vec<Vec<_>> = vec![];
//         assert_eq_map!(actual, expected);
//     }

//     #[test]
//     fn test_nplusone_nested_resolvers() {
//         let config = Config::default().query("Query").types(vec![
//             (
//                 "Query",
//                 Type::default().fields(vec![(
//                     "f1",
//                     Field::default()
//                         .type_of("F1".to_string())
//                         .into_list()
//                         .http(Http::default()),
//                 )]),
//             ),
//             (
//                 "F1",
//                 Type::default().fields(vec![(
//                     "f2",
//                     Field::default().type_of("F2".to_string()).into_list(),
//                 )]),
//             ),
//             (
//                 "F2",
//                 Type::default().fields(vec![(
//                     "f3",
//                     Field::default().type_of("F3".to_string()).into_list(),
//                 )]),
//             ),
//             (
//                 "F3",
//                 Type::default().fields(vec![(
//                     "f4",
//                     Field::default()
//                         .type_of("String".to_string())
//                         .http(Http::default()),
//                 )]),
//             ),
//         ]);

//         let actual = config.n_plus_one();
//         let expected = vec![vec![
//             ("Query", ("f1", "F1")),
//             ("F1", ("f2", "F2")),
//             ("F2", ("f3", "F3")),
//             ("F3", ("f4", "String")),
//         ]];

//         assert_eq_map!(actual, expected);
//     }

//     #[test]
//     fn test_nplusone_nested_resolvers_non_list_resolvers() {
//         let config = Config::default().query("Query").types(vec![
//             (
//                 "Query",
//                 Type::default().fields(vec![(
//                     "f1",
//                     Field::default()
//                         .type_of("F1".to_string())
//                         .http(Http::default()),
//                 )]),
//             ),
//             (
//                 "F1",
//                 Type::default().fields(vec![(
//                     "f2",
//                     Field::default().type_of("F2".to_string()).into_list(),
//                 )]),
//             ),
//             (
//                 "F2",
//                 Type::default().fields(vec![(
//                     "f3",
//                     Field::default().type_of("F3".to_string()).into_list(),
//                 )]),
//             ),
//             (
//                 "F3",
//                 Type::default().fields(vec![(
//                     "f4",
//                     Field::default()
//                         .type_of("String".to_string())
//                         .http(Http::default()),
//                 )]),
//             ),
//         ]);

//         let expected = vec![vec![
//             ("Query", ("f1", "F1")),
//             ("F1", ("f2", "F2")),
//             ("F2", ("f3", "F3")),
//             ("F3", ("f4", "String")),
//         ]];
//         let actual = config.n_plus_one();

//         assert_eq_map!(actual, expected);
//     }

//     #[test]
//     fn test_nplusone_nested_resolvers_without_resolvers() {
//         let config = Config::default().query("Query").types(vec![
//             (
//                 "Query",
//                 Type::default().fields(vec![(
//                     "f1",
//                     Field::default()
//                         .type_of("F1".to_string())
//                         .into_list()
//                         .http(Http::default()),
//                 )]),
//             ),
//             (
//                 "F1",
//                 Type::default().fields(vec![(
//                     "f2",
//                     Field::default().type_of("F2".to_string()).into_list(),
//                 )]),
//             ),
//             (
//                 "F2",
//                 Type::default()
//                     .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
//             ),
//         ]);

//         let expected: Vec<Vec<_>> = vec![];
//         let actual = config.n_plus_one();

//         assert_eq_map!(actual, expected);
//     }

//     #[test]
//     fn test_nplusone_cycles() {
//         let config = Config::default().query("Query").types(vec![
//             (
//                 "Query",
//                 Type::default().fields(vec![(
//                     "f1",
//                     Field::default()
//                         .type_of("F1".to_string())
//                         .into_list()
//                         .http(Http::default()),
//                 )]),
//             ),
//             (
//                 "F1",
//                 Type::default().fields(vec![
//                     ("f1", Field::default().type_of("F1".to_string())),
//                     ("f2", Field::default().type_of("F2".to_string()).into_list()),
//                 ]),
//             ),
//             (
//                 "F2",
//                 Type::default()
//                     .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
//             ),
//         ]);

//         let actual = config.n_plus_one();
//         let expected: Vec<Vec<_>> = vec![];

//         assert_eq_map!(actual, expected);
//     }

//     #[test]
//     fn test_nplusone_cycles_with_resolvers() {
//         let config = Config::default().query("Query").types(vec![
//             (
//                 "Query",
//                 Type::default().fields(vec![(
//                     "f1",
//                     Field::default()
//                         .type_of("F1".to_string())
//                         .into_list()
//                         .http(Http::default()),
//                 )]),
//             ),
//             (
//                 "F1",
//                 Type::default().fields(vec![
//                     ("f1", Field::default().type_of("F1".to_string()).into_list()),
//                     (
//                         "f2",
//                         Field::default()
//                             .type_of("String".to_string())
//                             .http(Http::default()),
//                     ),
//                 ]),
//             ),
//             (
//                 "F2",
//                 Type::default()
//                     .fields(vec![("f3", Field::default().type_of("String".to_string()))]),
//             ),
//         ]);

//         let actual = config.n_plus_one();
//         let expected = vec![
//             vec![
//                 ("Query", ("f1", "F1")),
//                 ("F1", ("f1", "F1")),
//                 ("F1", ("f2", "String")),
//             ],
//             vec![("Query", ("f1", "F1")), ("F1", ("f2", "String"))],
//         ];

//         assert_eq_map!(actual, expected);
//     }
//     #[test]
//     fn test_nplusone_nested_non_list() {
//         let f_field = Field::default()
//             .type_of("F".to_string())
//             .http(Http::default());

//         let config = Config::default().query("Query").types(vec![
//             ("Query", Type::default().fields(vec![("f", f_field)])),
//             (
//                 "F",
//                 Type::default().fields(vec![(
//                     "g",
//                     Field::default()
//                         .type_of("G".to_string())
//                         .into_list()
//                         .http(Http::default()),
//                 )]),
//             ),
//             (
//                 "G",
//                 Type::default().fields(vec![("e", Field::default().type_of("String".to_string()))]),
//             ),
//         ]);

//         let actual = config.n_plus_one();
//         let expected = Vec::<Vec<_>>::new();

//         assert_eq_map!(actual, expected);
//     }
// }
