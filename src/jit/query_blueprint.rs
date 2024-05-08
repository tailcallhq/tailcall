use async_graphql::parser::types::ExecutableDocument;

use crate::{
    blueprint::{Blueprint, Type},
    lambda::Expression,
};

struct Resolver {
    resolver: Option<Expression>,
    type_info: Type,
}

struct Field {
    name: String,
    selection: Selection,
    resolver: Resolver,
}

struct Selection {
    fields: Vec<Field>,
}

pub struct QueryBlueprint {
    selection: Selection,
}

impl QueryBlueprint {
    pub fn new(selection: ExecutableDocument, blueprint: &Blueprint) -> Self {
        todo!()
    }
}
