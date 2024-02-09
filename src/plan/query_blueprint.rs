use crate::lambda::Expression;
use crate::lambda::IO;
use crate::plan::query_plan::{Name, Node, QueryPlan, Resolver};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryBlueprint {
    root: NodeBlueprint,
}

impl From<QueryPlan> for QueryBlueprint {
    fn from(plan: QueryPlan) -> Self {
        QueryBlueprint { root: NodeBlueprint::from(plan.root) }
    }
}

impl Display for QueryBlueprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryBlueprint Tree:\n{}", self.root)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct NodeBlueprint {
    pub name: Name,
    pub id: u64,
    pub io_type: IOType,
    pub is_list: bool,
    pub children: Vec<NodeBlueprint>,
}
impl fmt::Display for NodeBlueprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn display_node(
            node: &NodeBlueprint,
            f: &mut fmt::Formatter<'_>,
            level: usize,
        ) -> fmt::Result {
            let indent = " ".repeat(level * 4);
            let branch = if level > 0 { "└── " } else { "" };
            let level_color = "\x1B[1;94m"; // Bright Blue for level, bold
            let foreach_color = "\x1B[1;95m"; // Bright Magenta for foreach label, bold
            let name_color = "\x1B[1;93m";
            let io_type_color = "\x1B[1;92m";
            let color_end = "\x1B[0m";

            // Display node name and attributes
            writeln!(
                f,
                "{}{}(Level {}){}{}{} #{}  {}{}",
                indent,
                level_color,
                level,
                branch,
                name_color,
                node.name,
                node.id,
                color_end,
                (node.io_type != IOType::Empty)
                    .then(|| format!(" {}- {}{}", io_type_color, node.io_type, color_end))
                    .unwrap_or_default(),
            )?;

            // Adjust "foreach" line indentation to start directly under the node name
            if node.is_list {
                writeln!(
                    f,
                    "{}    {}{}foreach{}",
                    indent, // Use the same indentation as the node
                    branch, // Reuse the branch symbol to align "foreach" correctly
                    foreach_color,
                    color_end
                )?;
            }

            // Recursively display child nodes
            for child in &node.children {
                display_node(child, f, level + 1)?;
            }

            Ok(())
        }

        display_node(self, f, 0)
    }
}

impl From<Node> for NodeBlueprint {
    fn from(node: Node) -> Self {
        NodeBlueprint {
            name: node.name,
            id: node.id,
            io_type: match node.expression {
                Resolver::Expression(expr) => match expr {
                    Expression::IO(io) => match io {
                        IO::Http { .. } => IOType::Http,
                        IO::Grpc { .. } => IOType::Grpc,
                        IO::GraphQL { .. } => IOType::Graphql,
                    },
                    _ => IOType::Empty,
                },
                Resolver::Empty => IOType::Empty,
            },
            is_list: node.is_list,
            children: node.children.into_iter().map(NodeBlueprint::from).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum IOType {
    Http,
    Grpc,
    Graphql,
    Empty,
}

impl fmt::Display for IOType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::Blueprint;
    use crate::config::{Config, ConfigModule};
    use crate::plan::query_blueprint::QueryBlueprint;
    use crate::valid::Validator;
    use async_graphql::parser::parse_query;
    use pretty_assertions::assert_eq;
    use serde_json::json;

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
        let blueprint = QueryBlueprint::from(plan);
        let expected = json!({
            "root": {
                "name": "Query",
                "id": 0,
                "io_type": "Empty",
                "is_list": false,
                "children": [
                    {
                        "name": "posts",
                        "id": 1,
                        "io_type": "Http",
                        "is_list": true,
                        "children": [
                            {
                                "name": "title",
                                "id": 2,
                                "io_type": "Empty",
                                "is_list": false,
                                "children": []
                            },
                            {
                                "name": "body",
                                "id": 3,
                                "io_type": "Empty",
                                "is_list": false,
                                "children": []
                            },
                            {
                                "name": "user",
                                "id": 4,
                                "io_type": "Http",
                                "is_list": false,
                                "children": [
                                    {
                                        "name": "name",
                                        "id": 5,
                                        "io_type": "Empty",
                                        "is_list": false,
                                        "children": []
                                    },
                                    {
                                        "name": "username",
                                        "id": 6,
                                        "io_type": "Empty",
                                        "is_list": false,
                                        "children": []
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        });
        let expected = serde_json::from_value::<QueryBlueprint>(expected).unwrap();

        assert_eq!(blueprint.to_string(), expected.to_string())
    }
}
