use async_graphql_value::Value;

use super::OperationPlan;
use crate::core::valid::{Valid, Validator};

mod query_complexity;
mod query_depth;

pub use query_complexity::QueryComplexity;
pub use query_depth::QueryDepth;

pub trait Rule {
    fn validate(&self, plan: &OperationPlan<Value>) -> Valid<(), String>;
}

pub trait RuleOps: Sized + Rule {
    fn pipe<Other: Rule>(self, other: Other) -> Pipe<Self, Other> {
        Pipe(self, other)
    }
}

impl<T: Rule> RuleOps for T {}

pub struct Pipe<A, B>(A, B);

impl<A, B> Rule for Pipe<A, B>
where
    A: Rule,
    B: Rule,
{
    fn validate(&self, plan: &OperationPlan<Value>) -> Valid<(), String> {
        self.0.validate(plan).and_then(|_| self.1.validate(plan))
    }
}
