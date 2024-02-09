use crate::lambda::Expression;
use crate::lambda::IO;
use crate::plan::query_plan::{Name, Node, QueryPlan, Resolver};
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
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

#[derive(Debug)]
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
            // Symbols and ANSI codes for enhanced display
            let indent = " ".repeat(level * 4); // Adjust spacing for clearer hierarchy
            let branch = if level > 0 { "└── " } else { " " }; // Branch symbol for child nodes
            let name_color = "\x1B[1;93m"; // Bright Yellow for names, bold
            let io_type_color = "\x1B[1;92m"; // Bright Green for IO types, bold
            let list_indicator_color = "\x1B[1;95m"; // Bright Magenta for list indicator, bold
            let color_end = "\x1B[0m"; // Reset to default

            // Compose the display string with vibrant colors and structured layout
            writeln!(
                f,
                "{}{}{}{} #{} {}{}{}",
                indent,
                branch,
                name_color,
                node.name,
                node.id,
                color_end,
                (node.io_type != IOType::Empty)
                    .then(|| format!(" {}- {}{}", io_type_color, node.io_type, color_end))
                    .unwrap_or_default(),
                node.is_list
                    .then(|| format!(" {}(List){}", list_indicator_color, color_end))
                    .unwrap_or_default()
            )?;

            for child in &node.children {
                display_node(child, f, level + 1)?;
            }

            Ok(())
        }

        // Start the recursive display with level 0
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

#[derive(Debug, PartialEq)]
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

        println!("{}", blueprint);
    }
}
