use indexmap::IndexMap;

use crate::core::config::{Config, Resolver};
use crate::core::valid::Valid;
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

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        let field_renames = self.0.iter().collect::<Vec<_>>();

        for (field_name, existing_and_suggested_names) in field_renames {
            let types_to_update = config
                .types
                .iter_mut()
                .filter(|(key, _)| *key == "Query" || *key == "Mutation")
                .map(|(_, type_)| type_)
                .collect::<Vec<_>>();

            for type_ in types_to_update {
                if let Some(mut field_info) = type_.fields.remove(field_name) {
                    for (existing_name, suggested_name) in existing_and_suggested_names {
                        if let Some(arg_value) = field_info.args.remove(existing_name) {
                            field_info
                                .args
                                .insert(suggested_name.to_string(), arg_value);

                            if let Some(Resolver::Http(http)) = field_info.resolver.as_mut() {
                                http.path = http.path.replace(existing_name, suggested_name);

                                for query in http.query.iter_mut() {
                                    query.key = query.key.replace(existing_name, suggested_name);
                                    query.value =
                                        query.value.replace(existing_name, suggested_name);
                                }

                                if let Some(body) = http.body.as_mut() {
                                    *body = body.replace(existing_name, suggested_name);
                                }
                            }

                            type_
                                .fields
                                .insert(field_name.to_string(), field_info.clone());
                        }
                    }
                }
            }
        }

        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {

    use maplit::hashmap;

    use super::RenameArgs;
    use crate::core::config::Config;
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    #[test]
    fn test_rename_args() {
        let sdl = r#"
            schema {
                query: Query
                mutation: Mutation
            }
            type User {
                id: ID!
                name: String
            }
            type Post {
                id: ID!
                title: String
                body: String
            }
            type Query {
                post(p1: ID!): Post @http(path: "/posts/{{args.p1}}")
            }
            type Mutation {
              createUser(p2: Int!): A @http(method: POST, path: "/users", body: "{{args.p2}}")
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let cfg = RenameArgs::new(
            hashmap! {
                "post".to_string() => vec![("p1".to_string(), "postId".to_string())],
                "createUser".to_string() => vec![("p2".to_string(), "userId".to_string())],
            }
            .into_iter(),
        )
        .transform(config)
        .to_result()
        .unwrap();

        insta::assert_snapshot!(cfg.to_sdl())
    }
}
