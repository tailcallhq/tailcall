use std::{collections::BTreeMap, fmt::Display, sync::Mutex};

use anyhow::{anyhow, Result};
use async_graphql::{Name, Value};
use dashmap::DashMap;
use futures_util::future::join_all;
use indexmap::IndexMap;

use crate::{
    http::RequestContext,
    lambda::{EvaluationContext, ResolverContextLike},
    query_plan::{
        plan::{GeneralPlan, OperationPlan},
        resolver::Id,
    },
};

use super::execution::ExecutionStep;

pub struct Executor<'a> {
    general_plan: &'a GeneralPlan,
    operation_plan: &'a OperationPlan,
}

pub struct ExecutionResult {
    resolved: BTreeMap<Id, Result<Value>>,
}

struct ExecutorContext<'a> {
    general_plan: &'a GeneralPlan,
    operation_plan: &'a OperationPlan,
    req_ctx: &'a RequestContext,
    resolved: DashMap<Id, Result<Value>>,
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
    #[async_recursion::async_recursion]
    pub async fn execute(&self, execution: &ExecutionStep) {
        match execution {
            ExecutionStep::Resolve(id) => {
                let field_plan = self.general_plan.field_plans.get(**id);

                let result = if let Some(field_plan) = field_plan {
                    let arguments = self.operation_plan.arguments_map.get(id);
                    // TODO: handle multiple parent values
                    let value = field_plan
                        .depends_on
                        .get(0)
                        .and_then(|id| self.resolved.get(id));
                    let value = value.as_ref().and_then(|v| v.value().as_ref().ok());
                    let graphql_ctx = GraphqlContext { arguments, value };
                    let eval_ctx = EvaluationContext::new(&self.req_ctx, &graphql_ctx);

                    field_plan.eval(eval_ctx).await
                } else {
                    Err(anyhow!("Failed to resolve field_plan for id: {id}"))
                };

                self.resolved.insert(*id, result);
            }
            ExecutionStep::ForEach(id) => {
                todo!()
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
    pub fn resolved(&mut self, id: &Id) -> Option<Result<Value>> {
        self.resolved.remove(id)
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

    fn add_error(&'a self, error: async_graphql::ServerError) {
        // TODO: add implementation
    }
}
