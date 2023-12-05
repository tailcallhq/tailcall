use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use async_graphql::parser::types::SelectionSet;
use indexmap::IndexMap;
use serde_json::Value;

use crate::blueprint::Blueprint;

// dumb Field representation
type Field = String;

// represents a wrapper for actual resolver that could manage what kind of fields
// we need to resolve for specific ResolvePlan and stores the result data
// inside to be able to read it later when building the response
#[derive(Hash, PartialEq, Eq, Clone)]
struct Resolver {
  // something to represent unique resolver tied with specific settings
  // to be able to reuse the same resolver to resolve different fields
  // i.e. for http is the upstream url with the path
  // and for graphql it's just upstream
  id: u32,
  // represent that this resolver depends on other resolvers that
  // should be executed before this on is called
  dependencies: Vec<u32>,
}

impl Resolver {
  // just for simplicity
  pub fn new(id: u32) -> Self {
    Self { id, dependencies: Vec::new() }
  }

  // mark that some field should be loaded with that resolver
  // actually it should be more complex to distinct different nested fields
  // for different types
  pub fn add_field(&mut self, field: Field) {
    todo!()
  }

  // something to resolve all the fields that came from the same resolver
  // TODO: nested fields
  pub fn resolve(&self, fields: HashSet<Field>) -> Value {
    todo!()
  }

  // get the resolved value for specific field
  pub fn data(&self, field: Field) -> Value {
    todo!()
  }
}

// Represents how every field could be resolved
// if the field is compound then recursively specify how its fields are resolved
// leafs are just some resolver and many leafs could shared the same resolver
enum ResolvePlan {
  Leaf(Resolver),
  Tree(IndexMap<String, ResolvePlan>),
}

// struct that could create ResolverPlan based on blueprint
// for specific set of selection fields
pub struct ResolverPlanner {}

impl ResolverPlanner {
  pub fn create_resolve_plan(&self, blueprint: Blueprint, fields: SelectionSet) -> (ResolvePlan, HashSet<Resolver>) {
    // collect all the resolvers that are used in this selection set
    let mut used_resolvers = HashSet::new();

    // some functions that walks over selection set of fields
    // and binds resolvers to fields
    self.walk(&mut used_resolvers, &"root".to_owned(), blueprint, fields);
    todo!()
  }

  fn walk(
    &self,
    fields_for_resolvers: &mut HashSet<Resolver>,
    field: &Field,
    blueprint: Blueprint,
    fields: SelectionSet,
  ) {
    todo!()
  }
}

// struct that accepts the ResolvePlan and used resolvers
// runs the resolvers
// and then combines the data based on ResolvePlan
pub struct ResolvePlanExecutor {}

impl ResolvePlanExecutor {
  pub fn run(&self, resolve_plan: ResolvePlan, used_resolvers: HashSet<Resolver>) -> Value {
    // combine all the resolvers with something like async DAG - https://crates.io/crates/async_dag
    // to properly manage order and run everything what possible in parallel
    todo!();

    // walk over ResolvePlan and gather the results from previously resolved resolvers data
    todo!();
  }
}

#[cfg(test)]
mod tests {
  use indexmap::indexmap;
  use serde_json::json;

  use super::*;

  #[test]
  fn test() {
    let blueprint = Blueprint::default();
    // imagine this selection set `users { id name post { id title body } }`
    let selection_set = SelectionSet { items: vec![] };

    let resolve_planner = ResolverPlanner {};
    let (resolve_plan, used_resolvers) = resolve_planner.create_resolve_plan(blueprint, selection_set);

    // imagine our resolve plan is resolved like this
    let resolve_plan: ResolvePlan = ResolvePlan::Tree(indexmap! {
      "users".to_owned() => ResolvePlan::Tree(indexmap! {
        "id".to_owned() => ResolvePlan::Leaf(Resolver::new(1)),
        "name".to_owned() => ResolvePlan::Leaf(Resolver::new(1)),
        "post".to_owned() => ResolvePlan::Tree(indexmap! {
          "id".to_owned() => ResolvePlan::Leaf(Resolver::new(3)),
          "title".to_owned() => ResolvePlan::Leaf(Resolver::new(3)),
          "body".to_owned() => ResolvePlan::Leaf(Resolver::new(4)),
        })
      })
    });

    // then run the executor and get the response
    let executor = ResolvePlanExecutor {};
    let result = executor.run(resolve_plan, used_resolvers);

    // and hoping to get this
    assert_eq!(
      result,
      json!({
        "users": [
          {
            "id": 1,
            "name": "Test Name",
            "post": {
              "id": 1,
              "title": "post title",
              "body": "how we migrated from scala to rust"
            }
          }
        ]
      })
    );
  }
}
