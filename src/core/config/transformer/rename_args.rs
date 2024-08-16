use indexmap::IndexMap;

use crate::core::config::Config;
use crate::core::valid::{Valid, Validator};
use crate::core::Transform;

/// A transformer that renames existing args by replacing them with suggested
/// names.
pub struct RenameArgs(IndexMap<String, Vec<(String, String)>>);

impl RenameArgs {
    pub fn new<I: Iterator<Item = (S, Vec<(S, S)>)>, S: ToString>(suggested_names: I) -> Self {
        Self(
            suggested_names
                .map(|(a, b)| {
                    (
                        a.to_string(),
                        b.into_iter()
                            .map(|(c, d)| (c.to_string(), d.to_string()))
                            .collect(),
                    )
                })
                .collect(),
        )
    }
}

impl Transform for RenameArgs {
    type Value = Config;
    type Error = String;

    fn transform(&self, config: Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut config = config;

        Valid::from_iter(
            self.0.iter(),
            |(field_name, existing_and_suggested_names)| {
                let query_type = config.types.get_mut("Query");
                if let Some(type_) = query_type {
                    if let Some(mut field_info) = type_.fields.remove(field_name) {
                        for (existing_name, suggested_name) in existing_and_suggested_names {
                            let arg_value = field_info.args.remove(existing_name);
                            if let Some(arg_value) = arg_value {
                                field_info
                                    .args
                                    .insert(suggested_name.to_string(), arg_value);
                                type_
                                    .fields
                                    .insert(field_name.to_string(), field_info.clone());
                            } else {
                                return Valid::fail(format!(
                                    "Failed to find argument '{}' in field {}.",
                                    existing_name, field_name
                                ));
                            }
                        }
                        Valid::succeed(())
                    } else {
                        Valid::fail(format!(
                            "Failed to find field '{}' in configuration.",
                            field_name
                        ))
                    }
                } else {
                    Valid::fail("Failed to find Query type in configuration.".to_string())
                }
            },
        )
        .map(|_| config)
    }
}

// #[cfg(test)]
// mod test {
//     use indexmap::IndexMap;
//     use maplit::hashmap;
//
//     use super::RenameArgs;
//     use crate::core::config::Config;
//     use crate::core::transform::Transform;
//     use crate::core::valid::{ValidationError, Validator};
//
//     #[test]
//     fn test_rename_type() {
//         let sdl = r#"
//             schema {
//                 query: Query
//             }
//             type A {
//                 id: ID!
//                 name: String
//             }
//             type Post {
//                 id: ID!
//                 title: String
//                 body: String
//             }
//             type B {
//                 name: String
//                 username: String
//             }
//             type Query {
//                 posts: [Post] @http(path: "/posts")
//             }
//             type Mutation {
//               createUser(user: B!): A @http(method: POST, path: "/users", body: "{{args.user}}")
//             }
//         "#;
//         let config = Config::from_sdl(sdl).to_result().unwrap();
//
//         let cfg = RenameArgs::new(
//             hashmap! {
//                 "Query" => "PostQuery",
//                 "A" => "User",
//                 "B" => "InputUser",
//                 "Mutation" => "UserMutation",
//             }
//             .iter(),
//         )
//         .transform(config)
//         .to_result()
//         .unwrap();
//
//         insta::assert_snapshot!(cfg.to_sdl())
//     }
//
//     #[test]
//     fn test_should_raise_error_when_operation_type_name_is_different() {
//         let sdl = r#"
//             schema {
//                 query: PostQuery
//             }
//             type Post {
//                 id: ID
//                 title: String
//             }
//             type PostQuery {
//                 posts: [Post] @http(path: "/posts")
//             }
//         "#;
//         let config = Config::from_sdl(sdl).to_result().unwrap();
//
//         let result = RenameArgs::new(hashmap! {"Query" =>  "PostQuery"}.iter())
//             .transform(config)
//             .to_result();
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn test_should_raise_error_when_type_not_found() {
//         let sdl = r#"
//             schema {
//                 query: Query
//             }
//             type A {
//                 id: ID!
//                 name: String
//             }
//             type Post {
//                 id: ID!
//                 title: String
//                 body: String
//             }
//             type Query {
//                 posts: [Post] @http(path: "/posts")
//             }
//         "#;
//         let config = Config::from_sdl(sdl).to_result().unwrap();
//
//         let mut suggested_names = IndexMap::new();
//         suggested_names.insert("Query", "PostQuery");
//         suggested_names.insert("A", "User");
//         suggested_names.insert("B", "User");
//         suggested_names.insert("C", "User");
//
//         let actual = RenameArgs::new(suggested_names.iter())
//             .transform(config)
//             .to_result();
//
//         let b_err = ValidationError::new("Type 'B' not found in configuration.".to_string());
//         let c_err = ValidationError::new("Type 'C' not found in configuration.".to_string());
//         let expected = Err(b_err.combine(c_err));
//         assert_eq!(actual, expected);
//     }
// }
