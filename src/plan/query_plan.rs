use async_graphql::parser::types::ExecutableDocument;

use crate::{blueprint::Blueprint, lambda::Expression};

pub struct Name(String);
pub struct Resolver {
    expression: Expression,
}
pub struct Node {
    name: Name,
    expression: Resolver,
    is_list: bool,
    is_required: bool,
    children: Vec<Node>,
    id: u64,
}

pub struct QueryPlan {
    root: Node,
}

impl Node {
    fn make(blueprint: Blueprint, document: ExecutableDocument) -> QueryPlan {
        // TODO: @shashitnakr:
        todo!()
    }
}
