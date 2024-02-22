use async_graphql::parser::types::{ExecutableDocument, SelectionSet};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::blueprint::Definition;
use crate::{blueprint::Blueprint, lambda::Expression};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Name(pub String);

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum Resolver {
    Expression(Expression),
    Empty,
}

#[derive(Debug)]
pub struct Node {
    pub(crate) name: Name,
    pub(crate) expression: Resolver,
    pub(crate) is_list: bool,
    is_required: bool,
    pub(crate) children: Vec<Node>,
    pub(crate) id: u64,
}

#[derive(Debug)]
pub struct QueryPlan {
    pub(crate) root: Node,
}

impl Node {
    fn recurse(
        blueprint: Blueprint,
        selection_set: &SelectionSet,
        name: &str,
        id_counter: u64,
    ) -> anyhow::Result<Vec<Node>> {
        let mut vec_node = Vec::new();
        let def = blueprint.definitions.iter().find_map(|def| {
            if def.name() == name {
                Some(def)
            } else {
                None
            }
        });
        let out = if let Some(def) = def {
            match def {
                Definition::Object(def) => {
                    let mut id_counter = id_counter;
                    for selection in selection_set.items.iter() {
                        match &selection.node {
                            async_graphql::parser::types::Selection::Field(field) => {
                                let p = def.fields.iter().find_map(|field_def| {
                                    if field_def.name == field.node.name.node {
                                        Some(field_def)
                                    } else {
                                        None
                                    }
                                });
                                if let Some(p) = p {
                                    let node = Node {
                                        name: Name(field.node.name.node.to_string()),
                                        expression: if let Some(resolver) = &p.resolver {
                                            Resolver::Expression(resolver.clone())
                                        } else {
                                            Resolver::Empty
                                        },
                                        is_list: p.of_type.is_list(),
                                        is_required: !p.of_type.is_nullable(),
                                        children: if !field.node.selection_set.node.items.is_empty()
                                        {
                                            Node::recurse(
                                                blueprint.clone(),
                                                &field.node.selection_set.node,
                                                &p.of_type.name(),
                                                id_counter + 1,
                                            )?
                                        } else {
                                            vec![]
                                        },
                                        id: id_counter,
                                    };
                                    vec_node.push(node);
                                    id_counter += 1;
                                }
                            }

                            _ => {}
                        }
                    }
                    Ok(vec_node)
                }
                _ => Err(anyhow::anyhow!("unsupported definition type")),
            }
        } else {
            Err(anyhow::anyhow!("definition not found"))
        };
        out
    }
    pub(crate) fn make(
        blueprint: Blueprint,
        document: ExecutableDocument,
    ) -> anyhow::Result<QueryPlan> {
        let mut vec_node = Vec::new();
        for (_, operation) in document.operations.iter() {
            let mut root_name = operation.node.ty.to_string();
            if root_name == "query" {
                root_name = "Query".to_string();
            }
            let selection_set = &operation.node.selection_set.node;
            vec_node = Node::recurse(blueprint.clone(), selection_set, &root_name, 1)?;
        }
        let root = Node {
            name: Name("Query".to_string()),
            expression: Resolver::Empty,
            is_list: false,
            is_required: false,
            children: vec_node,
            id: 0,
        };
        Ok(QueryPlan { root })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::Blueprint;
    use crate::config::{Config, ConfigModule};
    use crate::valid::Validator;
    use async_graphql::parser::parse_query;

    #[test]
    fn test_query_plan() {
        let config = r#"
            schema
              @server(port: 8000, graphiql: true, hostname: "0.0.0.0")
              @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: true, batch: {delay: 100}) {
              query: Query
            }

            type Query {
              posts: [Post] @http(path: "/posts")
              users: [User] @http(path: "/users")
              user(id: Int!): User @http(path: "/users/{{args.id}}")
            }

            type User {
              id: Int!
              name: String!
              username: String!
              email: String!
              phone: String
              website: String
            }

            type Post {
              id: Int!
              userId: Int!
              title: String!
              body: String!
              user: User @http(path: "/users/{{value.userId}}")
            }
        "#;
        let config = Config::from_sdl(config).to_result().unwrap();
        let config = ConfigModule::from(config);
        let blueprint = Blueprint::try_from(&config).unwrap();

        let document = parse_query(r#"{ posts {title body user {name username}} }"#).unwrap();
        let plan = Node::make(blueprint, document).unwrap();
        assert_eq!(plan.root.name.0, "Query");
        let posts_node = plan.root.children.iter().find(|&n| n.name.0 == "posts");
        assert!(
            posts_node.is_some(),
            "The 'posts' node is missing from the query plan."
        );
        // Further assertion to check if the "posts" node has a child named "title"
        let title_node = posts_node
            .unwrap()
            .children
            .iter()
            .find(|&n| n.name.0 == "title");
        assert!(
            title_node.is_some(),
            "The 'title' node is missing from the 'posts' node."
        );
    }
}
