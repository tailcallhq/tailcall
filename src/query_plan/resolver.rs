use crate::lambda::Expression;

use async_graphql::parser::types::SelectionSet;

pub struct Id(usize);

pub struct PlanResolver {
  id: Id,
  expression: Expression,
  selection_set: SelectionSet,
  depends_on: Vec<Id>
}


impl PlanResolver {
  pub fn new() -> Self {
    todo!()
  }
}