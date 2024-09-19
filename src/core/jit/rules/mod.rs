use super::OperationPlan;
use crate::core::valid::{Valid, Validator};

mod query_complexity;
mod query_depth;

pub use query_complexity::QueryComplexity;
pub use query_depth::QueryDepth;

pub trait ExecutionRule {
    fn validate<T: std::fmt::Debug>(&self, plan: &OperationPlan<T>) -> Valid<(), String>;
}

pub trait RuleOps: Sized + ExecutionRule {
    fn pipe<Other: ExecutionRule>(self, other: Other) -> Pipe<Self, Other> {
        Pipe(self, other)
    }
    fn when(self, cond: bool) -> When<Self> {
        When(self, cond)
    }
}

impl<T: ExecutionRule> RuleOps for T {}

pub struct Pipe<A, B>(A, B);

impl<A, B> ExecutionRule for Pipe<A, B>
where
    A: ExecutionRule,
    B: ExecutionRule,
{
    fn validate<T: std::fmt::Debug>(&self, plan: &OperationPlan<T>) -> Valid<(), String> {
        self.0.validate(plan).and_then(|_| self.1.validate(plan))
    }
}

pub struct When<A>(A, bool);

impl<A> ExecutionRule for When<A>
where
    A: ExecutionRule,
{
    fn validate<T: std::fmt::Debug>(&self, plan: &OperationPlan<T>) -> Valid<(), String> {
        if self.1 {
            self.0.validate(plan)
        } else {
            Valid::succeed(())
        }
    }
}

#[derive(Default)]
pub struct Rules;

impl ExecutionRule for Rules {
    fn validate<T: std::fmt::Debug>(&self, _: &OperationPlan<T>) -> Valid<(), String> {
        Valid::succeed(())
    }
}
