use std::collections::BTreeMap;
use std::fmt::Display;

use anyhow::{anyhow, Result};
use async_graphql::{Name, Value};
use dashmap::DashMap;
use futures_util::future::{join_all, try_join_all};
use indexmap::IndexMap;
use tailcall::core::http::RequestContext;
use tailcall::core::ir::{EvaluationContext, ResolverContextLike};

use super::step::ExecutionStep;
use crate::plan::{GeneralPlan, OperationPlan};
use crate::resolver::{FieldPlan, Id};

pub struct Executor<'a> {
    general_plan: &'a GeneralPlan,
    operation_plan: &'a OperationPlan,
}

#[derive(Debug)]
pub enum ResolvedEntry {
    Single(Result<Value>),
    List(Result<Vec<Value>>),
}

pub struct ExecutionResult {
    resolved: BTreeMap<Id, ResolvedEntry>,
}

struct ExecutorContext<'a> {
    general_plan: &'a GeneralPlan,
    operation_plan: &'a OperationPlan,
    req_ctx: &'a RequestContext,
    resolved: DashMap<Id, ResolvedEntry>,
}

impl Display for ExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.resolved)
    }
}

impl<'a> Executor<'a> {
    pub fn new(general_plan: &'a GeneralPlan, operation_plan: &'a OperationPlan) -> Self {
        Self { general_plan, operation_plan }
    }

    pub async fn execute(
        &self,
        req_ctx: &'a RequestContext,
        execution: &ExecutionStep,
    ) -> ExecutionResult {
        let executor_ctx = ExecutorContext {
            general_plan: self.general_plan,
            operation_plan: self.operation_plan,
            req_ctx,
            resolved: DashMap::new(),
        };

        executor_ctx.execute(execution).await;

        let resolved = executor_ctx.resolved.into_iter().collect();

        ExecutionResult { resolved }
    }
}

impl<'a> ExecutorContext<'a> {
    async fn eval(&self, field_plan: &FieldPlan, value: Option<&Value>) -> Result<Value> {
        let arguments = self.operation_plan.arguments_map.get(&field_plan.id);
        let graphql_ctx = GraphqlContext { arguments, value };
        let eval_ctx = EvaluationContext::new(self.req_ctx, &graphql_ctx);

        field_plan.eval(eval_ctx).await
    }

    #[async_recursion::async_recursion]
    pub async fn execute(&self, execution: &ExecutionStep) {
        match execution {
            ExecutionStep::Resolve(id) => {
                let field_plan = self
                    .general_plan
                    .field_plans
                    .get(**id)
                    .expect("Failed to resolved field_plan");

                let parent_field_plan_id = field_plan.depends_on.first();
                // TODO: handle multiple parent values
                let parent_resolved = parent_field_plan_id.and_then(|id| self.resolved.get(id));
                let parent_resolved = parent_resolved.as_ref().map(|v| v.value());

                // TODO: handle properly nesting for parent value since
                // rn it only considers child field is direct child of parent field
                let result = match parent_resolved {
                    Some(ResolvedEntry::List(Ok(list)))
                    | Some(ResolvedEntry::Single(Ok(Value::List(list)))) => {
                        let execution = list.iter().map(|value| self.eval(field_plan, Some(value)));

                        ResolvedEntry::List(try_join_all(execution).await)
                    }
                    Some(ResolvedEntry::List(Err(_err))) => {
                        ResolvedEntry::List(Err(anyhow!("Failed to resolve parent value")))
                    }
                    Some(ResolvedEntry::Single(value)) => {
                        ResolvedEntry::Single(self.eval(field_plan, value.as_ref().ok()).await)
                    }
                    None => ResolvedEntry::Single(self.eval(field_plan, None).await),
                };

                self.resolved.insert(*id, result);
            }
            ExecutionStep::Sequential(steps) => {
                for step in steps {
                    self.execute(step).await;
                }
            }
            ExecutionStep::Parallel(steps) => {
                join_all(steps.iter().map(|step| self.execute(step))).await;
            }
        }
    }
}

impl ExecutionResult {
    pub fn resolved(&self, id: &Id) -> Option<&ResolvedEntry> {
        self.resolved.get(id)
    }
}

#[derive(Clone)]
struct GraphqlContext<'a> {
    arguments: Option<&'a IndexMap<Name, Value>>,
    value: Option<&'a Value>,
}

impl<'a> ResolverContextLike<'a> for GraphqlContext<'a> {
    fn value(&'a self) -> Option<&'a Value> {
        self.value
    }

    fn args(&'a self) -> Option<&'a IndexMap<async_graphql::Name, Value>> {
        self.arguments
    }

    fn field(&'a self) -> Option<async_graphql::SelectionField> {
        None
    }

    fn add_error(&'a self, _error: async_graphql::ServerError) {
        // TODO: add implementation
    }
}
